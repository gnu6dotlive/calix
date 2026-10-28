//! The client side of the daemon's wire protocol.
//!
//! The daemon had no CLI client: the only thing that spoke this protocol was
//! the Rust frontend, so nothing on the command line could ask a question of
//! the process that owns the state. This is that client, kept small on
//! purpose — one request line out, one answer line back.
//!
//! It never writes state. A request is a request; the daemon remains the only
//! writer, which is the whole point of asking instead of editing a file.

const std = @import("std");

pub const protocol = @import("protocol.zig");

/// The daemon's answer, detached from the parse tree it arrived in.
///
/// `data` is the answer's JSON, already serialised: the daemon writes it that
/// way, and re-parsing it only to re-serialise would cost more than the
/// answer is worth.
pub const Answer = struct {
    ok: bool,
    /// The protocol's error code, when the request failed.
    code: ?[]const u8 = null,
    /// The daemon's sentence about why, when it wrote one.
    message: ?[]const u8 = null,
    /// The action's payload, serialised.
    data: ?[]const u8 = null,

    /// Releases what the answer owns.
    ///
    /// The answer is detached from the parse tree it arrived in, so the
    /// caller is what frees it -- and a caller that forgets holds the whole
    /// answer, not a pointer into a document it already dropped.
    pub fn deinit(self: Answer, allocator: std.mem.Allocator) void {
        if (self.code) |code| allocator.free(code);
        if (self.message) |message| allocator.free(message);
        if (self.data) |data| allocator.free(data);
    }
};

pub const Error = error{
    /// Nothing answered on the socket: absent, stale, or not a daemon.
    DaemonUnavailable,
    /// The socket path does not fit an AF_UNIX address.
    SocketPathTooLong,
    /// A line arrived but is not a response this client can read.
    MalformedAnswer,
    /// The daemon speaks a protocol version this client does not know.
    UnsupportedVersion,
    OutOfMemory,
};

/// The request line, exactly as it goes on the wire.
pub fn requestLine(
    allocator: std.mem.Allocator,
    action: protocol.Action,
) error{OutOfMemory}![]u8 {
    return std.fmt.allocPrint(
        allocator,
        "{{\"version\":{d},\"action\":\"{s}\"}}\n",
        .{ protocol.version, action.name() },
    );
}

/// Reads one response line.
///
/// The version is checked before the answer is believed, for the same reason
/// the daemon checks it on the way in: a client that reads a protocol it does
/// not know is a client that misreads it silently.
pub fn parseAnswer(allocator: std.mem.Allocator, line: []const u8) Error!Answer {
    const Wire = struct {
        version: u32 = protocol.version,
        ok: bool,
        @"error": ?[]const u8 = null,
        message: ?[]const u8 = null,
        data: ?std.json.Value = null,
    };

    var parsed = std.json.parseFromSlice(Wire, allocator, line, .{
        .ignore_unknown_fields = true,
    }) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.MalformedAnswer,
    };
    defer parsed.deinit();

    if (parsed.value.version != protocol.version) return error.UnsupportedVersion;

    var answer: Answer = .{ .ok = parsed.value.ok };
    if (parsed.value.@"error") |code| answer.code = try allocator.dupe(u8, code);
    if (parsed.value.message) |message| answer.message = try allocator.dupe(u8, message);
    if (parsed.value.data) |data| {
        if (data != .null) {
            answer.data = try std.json.Stringify.valueAlloc(allocator, data, protocol.json_options);
        }
    }
    return answer;
}

/// Sends one request and reads the answer.
///
/// A connect that fails is reported as `DaemonUnavailable` rather than
/// propagated with its syscall name: from the caller's side there is exactly
/// one thing to say about it, and it is not "FileNotFound" on a socket the
/// operator did not think of as a file.
pub fn exchange(
    allocator: std.mem.Allocator,
    io: std.Io,
    socket_path: []const u8,
    action: protocol.Action,
) Error!Answer {
    const address = std.Io.net.UnixAddress.init(socket_path) catch return error.SocketPathTooLong;

    var stream = address.connect(io) catch return error.DaemonUnavailable;
    defer stream.close(io);

    const line = try requestLine(allocator, action);
    defer allocator.free(line);

    var write_buffer: [256]u8 = undefined;
    var writer = stream.writer(io, &write_buffer);
    writer.interface.writeAll(line) catch return error.DaemonUnavailable;
    writer.interface.flush() catch return error.DaemonUnavailable;

    var read_buffer: [16 * 1024]u8 = undefined;
    var reader = stream.reader(io, &read_buffer);
    const answer = reader.interface.takeDelimiterInclusive('\n') catch return error.DaemonUnavailable;

    return parseAnswer(allocator, answer);
}

const testing = std.testing;

test "the request line carries the version and the action's wire name" {
    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();

    const line = try requestLine(arena.allocator(), .releases);
    try testing.expectEqualStrings("{\"version\":1,\"action\":\"releases\"}\n", line);

    // The name is the enum tag, so a rename is a protocol break. This is
    // where a client would notice that its daemon and it disagree.
    const decoded = try std.json.parseFromSlice(
        struct { version: u32, action: []const u8 },
        arena.allocator(),
        line,
        .{},
    );
    defer decoded.deinit();
    try testing.expectEqual(protocol.version, decoded.value.version);
    try testing.expectEqual(protocol.Action.releases, protocol.Action.fromName(decoded.value.action).?);
}

test "an answer's payload survives the parse tree it arrived in" {
    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();

    const answer = try parseAnswer(
        arena.allocator(),
        "{\"version\":1,\"ok\":true,\"data\":[{\"name\":\"a\"}]}\n",
    );
    try testing.expect(answer.ok);
    try testing.expectEqualStrings("[{\"name\":\"a\"}]", answer.data.?);
}

test "a failed answer keeps the daemon's code and sentence" {
    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();

    const answer = try parseAnswer(
        arena.allocator(),
        "{\"version\":1,\"ok\":false,\"error\":\"unknown-action\",\"message\":\"no such action\"}\n",
    );
    try testing.expect(!answer.ok);
    try testing.expectEqualStrings("unknown-action", answer.code.?);
    try testing.expectEqualStrings("no such action", answer.message.?);
    try testing.expect(answer.data == null);
}

test "an answer from a future protocol is refused, not misread" {
    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();

    try testing.expectError(
        error.UnsupportedVersion,
        parseAnswer(arena.allocator(), "{\"version\":2,\"ok\":true,\"data\":[]}"),
    );
}

test "an unknown field is ignored, so the daemon can add one" {
    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();

    const answer = try parseAnswer(
        arena.allocator(),
        "{\"version\":1,\"ok\":true,\"elapsed_ms\":12,\"data\":[]}",
    );
    try testing.expect(answer.ok);
}

test "a line that is not a response is named as such" {
    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();

    try testing.expectError(error.MalformedAnswer, parseAnswer(arena.allocator(), "not json"));
    try testing.expectError(error.MalformedAnswer, parseAnswer(arena.allocator(), "[]"));
}

test "an answer's owned strings are freed by deinit" {
    // The testing allocator fails the test if anything the answer owns
    // survives, which is the point: the parse tree is gone by then.
    const answer = try parseAnswer(
        testing.allocator,
        "{\"version\":1,\"ok\":false,\"error\":\"unknown-action\",\"message\":\"no such action\",\"data\":[1]}",
    );
    answer.deinit(testing.allocator);
}

test "an absent daemon is one error, not a syscall name" {
    var arena = std.heap.ArenaAllocator.init(testing.allocator);
    defer arena.deinit();

    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", arena.allocator());
    const socket_path = try std.fs.path.join(arena.allocator(), &.{ root, "daemon.sock" });

    try testing.expectError(
        error.DaemonUnavailable,
        exchange(arena.allocator(), testing.io, socket_path, .releases),
    );
}
