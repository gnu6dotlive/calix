//! `calix-notify` — the session-bus notification bridge.
//!
//! The package CLI sends `live.gnu6.calix.Refresh` after a package operation
//! that changed installed state. This service listens for that signal and
//! turns it into one desktop notification through
//! `org.freedesktop.Notifications`.
//!
//! It is deliberately separate from `calix-daemon`: the daemon is an
//! inventory service and must not depend on a desktop session. A missing
//! notification daemon is not an error for the package operation, and this
//! service is installed but not enabled by the package.

const std = @import("std");

const dbus = @cImport({
    @cInclude("dbus/dbus.h");
});

// dbus-errors.h describes DBusError with bitfields, which Zig's C translator
// demotes to an opaque type. The public layout is stable: two pointers, one
// unsigned int holding the placeholder bits, then a padding pointer. Keep it
// as an extern struct so dbus_error_init/free receive correctly sized storage.
const DBusErrorStorage = extern struct {
    name: [*c]const u8,
    message: [*c]const u8,
    dummy: c_uint,
    padding1: ?*anyopaque,
};

comptime {
    if (@sizeOf(DBusErrorStorage) != 32 or @alignOf(DBusErrorStorage) != 8) {
        @compileError("DBusErrorStorage no longer matches the x86_64 libdbus DBusError layout");
    }
}

const signal_interface = "live.gnu6.calix";
const signal_member = "Refresh";
const signal_path = "/live/gnu6/calix/Notifications";
const match_rule = "type='signal',interface='live.gnu6.calix',member='Refresh',path='/live/gnu6/calix/Notifications'";

pub fn main(init: std.process.Init) !void {
    const allocator = init.arena.allocator();
    const address = (try sessionBusAddress(allocator, init.environ_map)) orelse {
        std.debug.print(
            "calix-notify: no session bus (DBUS_SESSION_BUS_ADDRESS and XDG_RUNTIME_DIR are unset)\n",
            .{},
        );
        std.process.exit(1);
    };

    run(address) catch |err| {
        std.debug.print("calix-notify: {s}\n", .{@errorName(err)});
        std.process.exit(1);
    };
}

/// The address this process must use. An explicit address wins; otherwise
/// the per-user runtime directory is the only safe fallback. Returning null
/// is deliberate: a notification bridge must not autolaunch a bus on a
/// machine that has no desktop session.
fn sessionBusAddress(
    allocator: std.mem.Allocator,
    environment: *const std.process.Environ.Map,
) !?[:0]u8 {
    if (environment.get("DBUS_SESSION_BUS_ADDRESS")) |address| {
        if (address.len > 0) return try allocator.dupeZ(u8, address);
    }

    if (environment.get("XDG_RUNTIME_DIR")) |runtime_directory| {
        if (runtime_directory.len > 0 and std.fs.path.isAbsolute(runtime_directory)) {
            return try std.fmt.allocPrintSentinel(allocator, "unix:path={s}/bus", .{runtime_directory}, 0);
        }
    }

    return null;
}

fn run(address: [:0]const u8) !void {
    var error_storage: DBusErrorStorage = undefined;
    const dbus_error: *dbus.DBusError = @ptrCast(&error_storage);
    dbus.dbus_error_init(dbus_error);
    defer dbus.dbus_error_free(dbus_error);

    const connection = dbus.dbus_connection_open_private(address.ptr, dbus_error) orelse {
        reportDBusError("open session bus", &error_storage);
        return error.SessionBusUnavailable;
    };
    defer {
        dbus.dbus_connection_close(connection);
        dbus.dbus_connection_unref(connection);
    }

    if (dbus.dbus_bus_register(connection, dbus_error) == 0) {
        reportDBusError("register with session bus", &error_storage);
        return error.SessionBusRegistrationFailed;
    }

    // A dropped session bus should stop the process; systemd restarts it
    // within the unit's StartLimit window.
    dbus.dbus_connection_set_exit_on_disconnect(connection, 0);

    dbus.dbus_bus_add_match(connection, match_rule, dbus_error);
    if (dbus.dbus_error_is_set(dbus_error) != 0) {
        reportDBusError("subscribe to the Calix refresh signal", &error_storage);
        return error.SignalSubscriptionFailed;
    }
    dbus.dbus_connection_flush(connection);

    while (true) {
        if (dbus.dbus_connection_read_write_dispatch(connection, -1) == 0) {
            return error.SessionBusDisconnected;
        }

        while (dbus.dbus_connection_pop_message(connection)) |message| {
            defer dbus.dbus_message_unref(message);
            if (!isRefreshSignal(message)) continue;

            sendNotification(connection) catch |err| {
                std.debug.print("calix-notify: could not send notification: {s}\n", .{@errorName(err)});
            };
        }
    }
}

fn reportDBusError(operation: []const u8, error_storage: *const DBusErrorStorage) void {
    const dbus_error: *const dbus.DBusError = @ptrCast(error_storage);
    if (dbus.dbus_error_is_set(dbus_error) != 0) {
        std.debug.print("calix-notify: {s}: {s}: {s}\n", .{
            operation,
            cString(error_storage.name),
            cString(error_storage.message),
        });
    } else {
        std.debug.print("calix-notify: {s}: no error detail\n", .{operation});
    }
}

fn isRefreshSignal(message: *dbus.DBusMessage) bool {
    if (dbus.dbus_message_is_signal(message, signal_interface, signal_member) == 0) return false;
    const path = dbus.dbus_message_get_path(message);
    if (path == null) return false;
    return std.mem.eql(u8, cString(path), signal_path);
}

fn sendNotification(connection: *dbus.DBusConnection) !void {
    const message = try createNotificationMessage();
    defer dbus.dbus_message_unref(message);

    if (dbus.dbus_connection_send(connection, message, null) == 0) return error.SendFailed;
    dbus.dbus_connection_flush(connection);
}

fn createNotificationMessage() !*dbus.DBusMessage {
    const message = dbus.dbus_message_new_method_call(
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
    ) orelse return error.OutOfMemory;
    errdefer dbus.dbus_message_unref(message);

    var iter: dbus.DBusMessageIter = undefined;
    dbus.dbus_message_iter_init_append(message, &iter);

    try appendString(&iter, "Calix");
    try appendUint32(&iter, 0);
    try appendString(&iter, "live.gnu6.calix");
    try appendString(&iter, "Package state changed");
    try appendString(&iter, "Open Calix to review the updated package inventory.");

    var actions: dbus.DBusMessageIter = undefined;
    if (dbus.dbus_message_iter_open_container(&iter, dbus.DBUS_TYPE_ARRAY, "s", &actions) == 0)
        return error.OutOfMemory;
    if (dbus.dbus_message_iter_close_container(&iter, &actions) == 0)
        return error.OutOfMemory;

    var hints: dbus.DBusMessageIter = undefined;
    if (dbus.dbus_message_iter_open_container(&iter, dbus.DBUS_TYPE_ARRAY, "{sv}", &hints) == 0)
        return error.OutOfMemory;
    if (dbus.dbus_message_iter_close_container(&iter, &hints) == 0)
        return error.OutOfMemory;

    try appendInt32(&iter, 5000);
    return message;
}

fn appendString(iter: *dbus.DBusMessageIter, value: [*:0]const u8) !void {
    var pointer: [*c]const u8 = value;
    if (dbus.dbus_message_iter_append_basic(iter, dbus.DBUS_TYPE_STRING, @ptrCast(&pointer)) == 0)
        return error.OutOfMemory;
}

fn appendUint32(iter: *dbus.DBusMessageIter, value: dbus.dbus_uint32_t) !void {
    var copy = value;
    if (dbus.dbus_message_iter_append_basic(iter, dbus.DBUS_TYPE_UINT32, @ptrCast(&copy)) == 0)
        return error.OutOfMemory;
}

fn appendInt32(iter: *dbus.DBusMessageIter, value: dbus.dbus_int32_t) !void {
    var copy = value;
    if (dbus.dbus_message_iter_append_basic(iter, dbus.DBUS_TYPE_INT32, @ptrCast(&copy)) == 0)
        return error.OutOfMemory;
}

fn cString(value: [*c]const u8) []const u8 {
    if (value == null) return "";
    return std.mem.span(@as([*:0]const u8, @ptrCast(value)));
}

test "session bus address prefers the explicit environment and falls back to XDG" {
    var environment = std.process.Environ.Map.init(std.testing.allocator);
    defer environment.deinit();

    try std.testing.expect(try sessionBusAddress(std.testing.allocator, &environment) == null);

    try environment.put("XDG_RUNTIME_DIR", "/run/user/1000");
    const derived = (try sessionBusAddress(std.testing.allocator, &environment)).?;
    defer std.testing.allocator.free(derived);
    try std.testing.expectEqualStrings("unix:path=/run/user/1000/bus", derived);

    try environment.put("DBUS_SESSION_BUS_ADDRESS", "unix:abstract=/tmp/private-session");
    const explicit = (try sessionBusAddress(std.testing.allocator, &environment)).?;
    defer std.testing.allocator.free(explicit);
    try std.testing.expectEqualStrings("unix:abstract=/tmp/private-session", explicit);

    try environment.put("DBUS_SESSION_BUS_ADDRESS", "");
    try environment.put("XDG_RUNTIME_DIR", "relative/path");
    try std.testing.expect(try sessionBusAddress(std.testing.allocator, &environment) == null);
}

test "only the Calix Refresh signal is accepted" {
    const refresh = dbus.dbus_message_new_signal(
        signal_path,
        signal_interface,
        signal_member,
    ) orelse return error.OutOfMemory;
    defer dbus.dbus_message_unref(refresh);
    try std.testing.expect(isRefreshSignal(refresh));

    const other = dbus.dbus_message_new_signal(
        signal_path,
        signal_interface,
        "Other",
    ) orelse return error.OutOfMemory;
    defer dbus.dbus_message_unref(other);
    try std.testing.expect(!isRefreshSignal(other));

    const wrong_path = dbus.dbus_message_new_signal(
        "/live/gnu6/calix/Other",
        signal_interface,
        signal_member,
    ) orelse return error.OutOfMemory;
    defer dbus.dbus_message_unref(wrong_path);
    try std.testing.expect(!isRefreshSignal(wrong_path));
}

test "notification method call is a complete Notify message" {
    const message = try createNotificationMessage();
    defer dbus.dbus_message_unref(message);

    try std.testing.expectEqualStrings(
        "org.freedesktop.Notifications",
        cString(dbus.dbus_message_get_destination(message)),
    );
    try std.testing.expectEqualStrings(
        "/org/freedesktop/Notifications",
        cString(dbus.dbus_message_get_path(message)),
    );
    try std.testing.expectEqualStrings(
        "org.freedesktop.Notifications",
        cString(dbus.dbus_message_get_interface(message)),
    );
    try std.testing.expectEqualStrings("Notify", cString(dbus.dbus_message_get_member(message)));

    var iter: dbus.DBusMessageIter = undefined;
    try std.testing.expect(dbus.dbus_message_iter_init(message, &iter) != 0);

    try expectString(&iter, "Calix");
    try expectUint32(&iter, 0);
    try expectString(&iter, "live.gnu6.calix");
    try expectString(&iter, "Package state changed");
    try expectString(&iter, "Open Calix to review the updated package inventory.");

    try std.testing.expectEqual(dbus.DBUS_TYPE_ARRAY, dbus.dbus_message_iter_get_arg_type(&iter));
    try std.testing.expect(dbus.dbus_message_iter_next(&iter) != 0);
    try std.testing.expectEqual(dbus.DBUS_TYPE_ARRAY, dbus.dbus_message_iter_get_arg_type(&iter));
    try std.testing.expect(dbus.dbus_message_iter_next(&iter) != 0);
    try std.testing.expectEqual(dbus.DBUS_TYPE_INT32, dbus.dbus_message_iter_get_arg_type(&iter));

    var timeout: dbus.dbus_int32_t = 0;
    dbus.dbus_message_iter_get_basic(&iter, @ptrCast(&timeout));
    try std.testing.expectEqual(@as(dbus.dbus_int32_t, 5000), timeout);
    try std.testing.expect(dbus.dbus_message_iter_next(&iter) == 0);
}

fn expectString(iter: *dbus.DBusMessageIter, expected: []const u8) !void {
    try std.testing.expectEqual(dbus.DBUS_TYPE_STRING, dbus.dbus_message_iter_get_arg_type(iter));
    var value: [*c]const u8 = null;
    dbus.dbus_message_iter_get_basic(iter, @ptrCast(&value));
    try std.testing.expectEqualStrings(expected, cString(value));
    try std.testing.expect(dbus.dbus_message_iter_next(iter) != 0);
}

fn expectUint32(iter: *dbus.DBusMessageIter, expected: dbus.dbus_uint32_t) !void {
    try std.testing.expectEqual(dbus.DBUS_TYPE_UINT32, dbus.dbus_message_iter_get_arg_type(iter));
    var value: dbus.dbus_uint32_t = 0;
    dbus.dbus_message_iter_get_basic(iter, @ptrCast(&value));
    try std.testing.expectEqual(expected, value);
    try std.testing.expect(dbus.dbus_message_iter_next(iter) != 0);
}
