//! The daemon's wire protocol.
//!
//! One JSON object per line in each direction, the same convention the GUI's
//! control socket already uses: a request is
//! `{"version":1,"action":"inventory","payload":...}` and a response is
//! `{"version":1,"ok":true,"data":...}`.
//!
//! The version is checked before the action is dispatched. A client that
//! speaks a version this daemon does not know is told so, rather than
//! having its request silently misread — which is the only alternative when
//! a protocol has no version.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

/// The protocol version.
pub const version: u32 = 1;

/// The document convention shared with the inventory.
pub const json_options = Zigalpm.shared.json.omit_null_optionals;

pub const Action = enum {
    ping,
    status,
    inventory,
    releases,
    @"mark-notified",

    /// The action a name denotes, exactly.
    ///
    /// An unknown action is a distinct answer from a malformed request, so
    /// this parses the name rather than letting the JSON decoder reject the
    /// whole line.
    pub fn fromName(text: []const u8) ?Action {
        inline for (@typeInfo(Action).@"enum".fields) |field| {
            if (std.mem.eql(u8, text, field.name)) return @enumFromInt(field.value);
        }
        return null;
    }

    pub fn name(self: Action) []const u8 {
        return @tagName(self);
    }
};

/// Why a request did not succeed.
pub const ErrorCode = enum {
    /// The line is not a request this daemon can read.
    @"bad-request",
    /// The request speaks a protocol version this daemon does not know.
    @"unsupported-version",
    /// The action is not one this daemon serves.
    @"unknown-action",
    /// The action is known but failed.
    failed,

    pub fn name(self: ErrorCode) []const u8 {
        return @tagName(self);
    }
};

/// A request as it arrives.
pub const Request = struct {
    version: u32 = version,
    action: []const u8,
    payload: ?std.json.Value = null,
};

/// Writes a response line.
///
/// `data` is spliced in already serialised: the inventory document is large,
/// and parsing it back into a `std.json.Value` only to re-serialise it would
/// cost more than the whole request.
pub fn writeResponse(
    writer: *std.Io.Writer,
    ok: bool,
    code: ?ErrorCode,
    message: ?[]const u8,
    data: ?[]const u8,
) !void {
    try writer.writeAll("{\"version\":");
    try writer.print("{d}", .{version});
    try writer.writeAll(",\"ok\":");
    try writer.writeAll(if (ok) "true" else "false");
    if (code) |value| {
        try writer.writeAll(",\"error\":");
        try std.json.Stringify.value(value.name(), json_options, writer);
    }
    if (message) |value| {
        try writer.writeAll(",\"message\":");
        try std.json.Stringify.value(value, json_options, writer);
    }
    try writer.writeAll(",\"data\":");
    try writer.writeAll(data orelse "null");
    try writer.writeAll("}\n");
}

const testing = std.testing;

test "Action: names round-trip, and the kebab-case action parses" {
    try testing.expectEqual(Action.ping, Action.fromName("ping").?);
    try testing.expectEqual(Action.inventory, Action.fromName("inventory").?);
    try testing.expectEqual(Action.@"mark-notified", Action.fromName("mark-notified").?);
    try testing.expectEqualStrings("mark-notified", Action.@"mark-notified".name());
    try testing.expect(Action.fromName("inventor") == null);
    try testing.expect(Action.fromName("") == null);
    // The tag name is the wire name: a rename would be a protocol break, and
    // this is where it would be noticed.
    inline for (@typeInfo(Action).@"enum".fields) |field| {
        const action: Action = @enumFromInt(field.value);
        try testing.expectEqualStrings(field.name, action.name());
    }
}

test "writeResponse: a successful response carries data and no error" {
    var buffer: [256]u8 = undefined;
    var writer = std.Io.Writer.fixed(&buffer);
    try writeResponse(&writer, true, null, "done", "{\"a\":1}");

    const line = writer.buffered();
    try testing.expect(std.mem.indexOf(u8, line, "\"version\":1") != null);
    try testing.expect(std.mem.indexOf(u8, line, "\"ok\":true") != null);
    try testing.expect(std.mem.indexOf(u8, line, "\"data\":{\"a\":1}") != null);
    try testing.expect(std.mem.indexOf(u8, line, "error") == null);
    try testing.expect(std.mem.endsWith(u8, line, "\n"));
}

test "writeResponse: a failure names its code" {
    var buffer: [256]u8 = undefined;
    var writer = std.Io.Writer.fixed(&buffer);
    try writeResponse(&writer, false, .@"unsupported-version", "this daemon speaks 1", null);

    const line = writer.buffered();
    try testing.expect(std.mem.indexOf(u8, line, "\"ok\":false") != null);
    try testing.expect(std.mem.indexOf(u8, line, "\"error\":\"unsupported-version\"") != null);
    try testing.expect(std.mem.indexOf(u8, line, "\"data\":null") != null);
}

test "writeResponse: a message is escaped, not interpolated" {
    var buffer: [256]u8 = undefined;
    var writer = std.Io.Writer.fixed(&buffer);
    try writeResponse(&writer, false, .failed, "a \"quoted\" message", null);

    const line = writer.buffered();
    try testing.expect(std.mem.indexOf(u8, line, "a \\\"quoted\\\" message") != null);
}

test "writeResponse: an absent message is omitted, not written as null" {
    var buffer: [256]u8 = undefined;
    var writer = std.Io.Writer.fixed(&buffer);
    try writeResponse(&writer, true, null, null, "[]");

    const line = writer.buffered();
    try testing.expect(std.mem.indexOf(u8, line, "message") == null);
}
