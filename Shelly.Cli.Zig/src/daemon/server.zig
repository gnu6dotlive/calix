//! `calix-daemon`: the one writer of Calix state.
//!
//! The frontend never reads a state file. It asks the daemon, and the daemon
//! is the only process that writes: two writers on one file means one of
//! them silently loses a decision, and "I already told the user about 2.1.0"
//! is exactly the kind of decision that must not be lost.
//!
//! The daemon serves four things:
//!
//!   * `inventory` — every ecosystem, normalised into one versioned document;
//!   * `releases`  — probe each update source and record what it offered;
//!   * `mark-notified` — record that the user has been told, so they are not
//!     told twice;
//!   * `ping` and `status` — whether it is alive, and what it is doing.
//!
//! Connections are served one at a time. A second client waits rather than
//! racing: the daemon holds mutable state, and serialising requests is how
//! that state stays coherent. The inventory takes a few hundred milliseconds
//! on a machine with two thousand packages; a queue of one is not a
//! bottleneck worth the correctness cost of concurrency.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const runtime = @import("../runtime/context.zig");
const xdg = @import("../runtime/xdg.zig");
const service = @import("../inventory/service.zig");
const protocol = @import("protocol.zig");
const journal_mod = @import("journal.zig");
const config_manager = @import("../config/manager.zig");

const release_registry = Zigalpm.release.registry;

pub const Error = error{
    /// Another daemon is already serving this socket.
    AlreadyRunning,
};

pub const Options = struct {
    socket_path: []const u8,
    state_directory: []const u8,
};

pub const Server = struct {
    /// Long-lived: the registry lives here.
    allocator: std.mem.Allocator,
    io: std.Io,
    /// The base context. Each request gets a copy whose allocator is a
    /// per-request arena, so a big inventory does not accumulate.
    context: *runtime.RuntimeContext,
    options: Options,
    journal: journal_mod.Journal,
    registry: release_registry.Registry,
    registry_path: []const u8,
    /// When this process started, for `status`.
    started_at: i64,
    /// The same moment on the monotonic clock, for the uptime.
    started_monotonic: std.Io.Timestamp,

    /// Prepares a daemon.
    ///
    /// The registry is loaded here, once: a request that had to read it
    /// would race with the write that follows it.
    pub fn init(
        allocator: std.mem.Allocator,
        io: std.Io,
        context: *runtime.RuntimeContext,
        options: Options,
    ) !Server {
        // Both of these outlive `init`: the journal keeps its directory and
        // the server keeps the registry path. Freeing either here would leave
        // the server appending to freed memory, which is exactly the bug that
        // made the journal silently disappear.
        const journal_directory = try std.fs.path.join(allocator, &.{ options.state_directory, "journal" });
        errdefer allocator.free(journal_directory);
        // Not `releases.json`: that name already belongs to the frozen
        // engine's release observations in the same directory, and a
        // registry written over it would destroy them.
        const registry_path = try std.fs.path.join(allocator, &.{ options.state_directory, "release-registry.json" });
        errdefer allocator.free(registry_path);

        return .{
            .allocator = allocator,
            .io = io,
            .context = context,
            .options = options,
            .journal = journal_mod.Journal.init(allocator, io, journal_directory),
            .registry = try release_registry.Registry.load(allocator, io, registry_path),
            .registry_path = registry_path,
            .started_at = now(io),
            .started_monotonic = std.Io.Timestamp.now(io, .awake),
        };
    }

    pub fn deinit(self: *Server) void {
        self.registry.deinit();
        self.allocator.free(self.registry_path);
        self.allocator.free(self.journal.directory);
        self.* = undefined;
    }

    /// Serves until the process is killed.
    pub fn run(self: *Server) !void {
        if (std.fs.path.dirname(self.options.socket_path)) |directory| {
            var handle = try std.Io.Dir.cwd().createDirPathOpen(self.io, directory, .{
                .permissions = .fromMode(0o700),
            });
            handle.close(self.io);
        }

        const address = try std.Io.net.UnixAddress.init(self.options.socket_path);
        if (try isServing(self.io, &address)) return Error.AlreadyRunning;
        // Nothing is listening, so a socket file here is the leftover of a
        // daemon that died. Removing it is safe; leaving it would make every
        // later start fail with an address that answers nothing.
        std.Io.Dir.cwd().deleteFile(self.io, self.options.socket_path) catch {};

        var server = try address.listen(self.io, .{});
        defer {
            server.deinit(self.io);
            std.Io.Dir.cwd().deleteFile(self.io, self.options.socket_path) catch {};
        }

        // The socket is user-private: it serves the user's own state, and
        // another user has no business asking.
        std.Io.Dir.cwd().setFilePermissions(
            self.io,
            self.options.socket_path,
            .fromMode(0o600),
            .{},
        ) catch {};

        self.journal.append(.{
            .at = now(self.io),
            .action = "daemon",
            .ok = true,
            .message = "listening",
        });

        while (true) {
            var stream = server.accept(self.io) catch |err| switch (err) {
                error.ConnectionAborted => continue,
                else => return err,
            };
            self.serveConnection(&stream);
        }
    }

    fn serveConnection(self: *Server, stream: *std.Io.net.Stream) void {
        defer stream.close(self.io);

        var read_buffer: [16 * 1024]u8 = undefined;
        var reader = stream.reader(self.io, &read_buffer);
        var write_buffer: [16 * 1024]u8 = undefined;
        var writer = stream.writer(self.io, &write_buffer);

        while (true) {
            // The line lives in the reader's buffer and stays valid until
            // the next read, so the response is written before reading again.
            const line = reader.interface.takeDelimiterInclusive('\n') catch return;
            self.handleLine(&writer.interface, line) catch return;
            writer.interface.flush() catch return;
        }
    }

    fn handleLine(self: *Server, writer: *std.Io.Writer, line: []const u8) !void {
        const trimmed = std.mem.trim(u8, line, " \t\r\n");
        if (trimmed.len == 0) return;

        // One arena per request: the inventory allocates a record per
        // installed package, and none of it outlives the response.
        var arena = std.heap.ArenaAllocator.init(self.allocator);
        defer arena.deinit();
        const allocator = arena.allocator();

        const parsed = std.json.parseFromSlice(protocol.Request, allocator, trimmed, .{
            .ignore_unknown_fields = true,
        }) catch {
            try protocol.writeResponse(
                writer,
                false,
                .@"bad-request",
                "The request is not a JSON object this daemon can read.",
                null,
            );
            self.journal.append(.{ .at = now(self.io), .action = "?", .ok = false, .message = "bad request" });
            return;
        };
        const request = parsed.value;

        if (request.version != protocol.version) {
            try protocol.writeResponse(
                writer,
                false,
                .@"unsupported-version",
                "This daemon speaks protocol version 1.",
                null,
            );
            self.journal.append(.{
                .at = now(self.io),
                .action = request.action,
                .ok = false,
                .message = "unsupported protocol version",
            });
            return;
        }

        const action = protocol.Action.fromName(request.action) orelse {
            try protocol.writeResponse(writer, false, .@"unknown-action", "No such action.", null);
            self.journal.append(.{
                .at = now(self.io),
                .action = request.action,
                .ok = false,
                .message = "unknown action",
            });
            return;
        };

        const data = self.dispatch(allocator, action, request.payload) catch |err| {
            try protocol.writeResponse(writer, false, .failed, @errorName(err), null);
            self.journal.append(.{
                .at = now(self.io),
                .action = action.name(),
                .ok = false,
                .message = @errorName(err),
            });
            return;
        };

        try protocol.writeResponse(writer, true, null, null, data);
        self.journal.append(.{ .at = now(self.io), .action = action.name(), .ok = true });
    }

    /// Serves one action and returns its serialised `data`.
    fn dispatch(
        self: *Server,
        allocator: std.mem.Allocator,
        action: protocol.Action,
        payload: ?std.json.Value,
    ) ![]const u8 {
        return switch (action) {
            .ping => std.json.Stringify.valueAlloc(allocator, .{
                .pid = std.os.linux.getpid(),
                .protocol = protocol.version,
                .started_at = self.started_at,
            }, protocol.json_options),

            .status => std.json.Stringify.valueAlloc(allocator, .{
                .pid = std.os.linux.getpid(),
                .protocol = protocol.version,
                .started_at = self.started_at,
                .uptime_seconds = uptimeSeconds(self.io, self.started_monotonic),
                .socket_path = self.options.socket_path,
                .state_directory = self.options.state_directory,
                .tracked_releases = self.registry.entries.items.len,
            }, protocol.json_options),

            .inventory => {
                var request_context = self.context.*;
                request_context.allocator = allocator;
                var snapshot = try service.snapshot(&request_context, .{
                    .state_directory = self.options.state_directory,
                });
                defer snapshot.deinit();
                return try snapshot.toJson();
            },

            .releases => blk: {
                var request_context = self.context.*;
                request_context.allocator = allocator;
                try self.probeReleases(&request_context);
                var reported: std.ArrayList(ReportedRelease) = .empty;
                for (self.registry.entries.items) |entry| {
                    try reported.append(allocator, .{
                        .name = entry.name,
                        .source = entry.source,
                        .seen = entry.last_seen,
                        .notified = entry.notified,
                        .answered = entry.last_error == null,
                        .consecutive_failures = entry.consecutive_failures,
                        .is_news = entry.hasNews(),
                    });
                }
                break :blk try std.json.Stringify.valueAlloc(allocator, reported.items, protocol.json_options);
            },

            .@"mark-notified" => blk: {
                const object = payload orelse return error.MissingPayload;
                if (object != .object) return error.MalformedPayload;
                const name = object.object.get("name") orelse return error.MalformedPayload;
                const version = object.object.get("version") orelse return error.MalformedPayload;
                if (name != .string or version != .string) return error.MalformedPayload;
                try self.registry.markNotified(name.string, version.string);
                try self.registry.save(self.io, self.registry_path);
                break :blk try std.json.Stringify.valueAlloc(allocator, .{
                    .name = name.string,
                    .notified = version.string,
                }, protocol.json_options);
            },
        };
    }

    /// Probes every update source and records what it offered.
    ///
    /// An error from a source is recorded as a failed probe and the loop
    /// continues: one forge being down must not hide the releases the
    /// others are offering.
    fn probeReleases(self: *Server, context: *runtime.RuntimeContext) !void {
        const allocator = context.allocator;
        const configuration = config_manager.Manager.init(context).read() catch null;
        const install_directory = if (configuration) |value|
            stringValue(&value, "AppImageInstallPath") orelse try xdg.binHome(context)
        else
            try xdg.binHome(context);
        const local_db_path = try std.fs.path.join(
            allocator,
            &.{ try xdg.configHome(context), "shelly", "appimage-metadata-v2.db" },
        );

        var apps_manager = Zigalpm.AppImageManager{
            .allocator = allocator,
            .io = self.io,
            .environ = context.environ,
            .install_directory = install_directory,
            .local_db_path = local_db_path,
        };
        defer apps_manager.deinit();

        const apps = try apps_manager.getAppImagesFromLocalDb();
        defer apps_manager.freeAppImages(apps);

        var update_manager = Zigalpm.appimage.UpdateManager{
            .allocator = allocator,
            .io = self.io,
            .environ = context.environ,
            .install_directory = install_directory,
            .local_db_path = local_db_path,
        };
        defer update_manager.deinit();

        const at = now(self.io);
        for (apps) |*app| {
            // An AppImage with no update source is not tracked: there is
            // nothing to probe and nothing to tell the user about.
            if (app.update_type == .none) continue;
            const source = @tagName(app.update_type);

            const maybe_update = update_manager.get_update(app) catch |err| {
                _ = try self.registry.record(app.name, source, .{ .failed = @errorName(err) }, at);
                continue;
            };
            const update = maybe_update orelse continue;
            var owned = update;
            defer owned.deinit(allocator);

            _ = try self.registry.record(app.name, source, .{ .seen = owned.version }, at);
        }

        try self.registry.save(self.io, self.registry_path);
    }
};

/// A registry entry as the frontend sees it.
const ReportedRelease = struct {
    name: []const u8,
    source: []const u8,
    seen: ?[]const u8 = null,
    notified: ?[]const u8 = null,
    answered: bool,
    consecutive_failures: u32,
    is_news: bool,
};

/// Whether a daemon is already answering on this socket.
///
/// A connect that succeeds means a live daemon; any failure means the socket
/// is stale or absent, and a stale socket is safe to replace.
fn isServing(io: std.Io, address: *const std.Io.net.UnixAddress) !bool {
    const stream = address.connect(io) catch return false;
    stream.close(io);
    return true;
}

/// Wall-clock seconds, for anything a human reads or that is compared
/// across restarts.
///
/// `.real` is the Unix epoch. `.awake` is monotonic since boot: correct for
/// measuring an interval, wrong for recording when something happened.
fn now(io: std.Io) i64 {
    return @intCast(@divTrunc(std.Io.Timestamp.now(io, .real).nanoseconds, std.time.ns_per_s));
}

/// Monotonic seconds, for measuring how long this process has been up.
///
/// A wall-clock jump backwards must not turn an uptime negative.
fn uptimeSeconds(io: std.Io, since: std.Io.Timestamp) i64 {
    return @intCast(@divTrunc(
        since.durationTo(std.Io.Timestamp.now(io, .awake)).nanoseconds,
        std.time.ns_per_s,
    ));
}

fn stringValue(configuration: *const @import("../config/model.zig").Config, key: []const u8) ?[]const u8 {
    const value = configuration.values.get(key) orelse return null;
    if (value != .string or value.string.len == 0) return null;
    return value.string;
}

const testing = std.testing;

test "isServing: an absent socket is not a running daemon" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    const path = try std.fs.path.join(testing.allocator, &.{ buffer[0..length], "absent.sock" });
    defer testing.allocator.free(path);

    const address = try std.Io.net.UnixAddress.init(path);
    try testing.expect(!try isServing(testing.io, &address));
}

test "isServing: a bound socket with no daemon behind it is not a running daemon" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    const path = try std.fs.path.join(testing.allocator, &.{ buffer[0..length], "stale.sock" });
    defer testing.allocator.free(path);

    // Create the file without listening: this is what a daemon leaves behind
    // when it dies without cleaning up.
    var file = try std.Io.Dir.cwd().createFile(testing.io, path, .{});
    file.close(testing.io);

    const address = try std.Io.net.UnixAddress.init(path);
    try testing.expect(!try isServing(testing.io, &address));
}
