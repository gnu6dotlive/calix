//! `calix releases` — what is release-tracked, and what was last seen.
//!
//! Reading does not need the daemon. The document is on disk, and a command
//! that cannot answer while a service is down is a command that hides state
//! the machine already has. Reading is also the only thing this command does
//! without help: the document belongs to the daemon, and a CLI that wrote it
//! would be a second writer behind a lock — the fault this workstream exists
//! to remove.
//!
//! `--refresh` therefore asks. The request goes over the daemon's socket, the
//! daemon probes and commits, and the answer is what it committed.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const runtime = @import("../runtime/context.zig");
const parser = @import("../cli/parser.zig");
const spec = @import("../cli/spec.zig");
const xdg = @import("../runtime/xdg.zig");
const address = @import("../daemon/address.zig");
const client = @import("../daemon/client.zig");

const tracked = Zigalpm.release.tracked;

const command_path = "calix releases standard";

/// The largest document read. The same bound the inventory applies, for the
/// same reason: a corrupted file must not be read into memory whole.
const max_document_bytes = 8 * 1024 * 1024;

/// Whether this command owns the invocation.
///
/// The dispatcher asks every command in order whether it owns the
/// invocation, so a command that answered for everything would replace
/// every command that comes after it in the list.
fn claims(invocation: *const parser.Invocation) bool {
    return std.mem.eql(u8, invocation.command.path, command_path);
}

pub fn dispatch(
    context: *runtime.RuntimeContext,
    invocation: *const parser.Invocation,
) !?u8 {
    if (!claims(invocation)) return null;
    if (optionEnabled(invocation, "--refresh")) return try refresh(context);
    return try printCommitted(context);
}

/// Prints the document as the daemon committed it.
///
/// The bytes are passed through rather than re-rendered: the CLI is not the
/// owner, and a renderer here would be a second opinion about a document that
/// has exactly one writer.
fn printCommitted(context: *runtime.RuntimeContext) !u8 {
    const path = try documentPath(context);
    defer context.allocator.free(path);

    const text = std.Io.Dir.cwd().readFileAlloc(
        context.io,
        path,
        context.allocator,
        .limited(max_document_bytes),
    ) catch |err| switch (err) {
        // Nothing tracked yet is a state, not a fault: the document is an
        // observation, and no observation has been made.
        error.FileNotFound => {
            try context.stdout.writeAll("{}\n");
            return 0;
        },
        error.OutOfMemory => return error.OutOfMemory,
        else => {
            try context.stderr.print("releases: cannot read {s}\n", .{path});
            return 1;
        },
    };
    defer context.allocator.free(text);

    // Validated before printing. The bytes come from another writer, and a
    // reader that prints whatever is on disk cannot tell a corrupted document
    // from an empty one.
    var parsed = std.json.parseFromSlice(std.json.Value, context.allocator, text, .{}) catch {
        try context.stderr.print(
            "releases: {s} is not JSON; the daemon's next probe replaces it\n",
            .{path},
        );
        return 1;
    };
    parsed.deinit();

    try context.stdout.writeAll(std.mem.trimEnd(u8, text, " \t\r\n"));
    try context.stdout.writeAll("\n");
    return 0;
}

/// Asks the daemon to probe now, and prints what it committed.
fn refresh(context: *runtime.RuntimeContext) !u8 {
    const socket_path = try address.socketPath(context.allocator, context);
    defer context.allocator.free(socket_path);

    const answer = client.exchange(context.allocator, context.io, socket_path, .releases) catch |err| {
        switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            error.DaemonUnavailable => {
                try context.stderr.print(
                    "releases: no daemon is answering on {s}. A refresh is a write, and the daemon is the writer; start it with `systemctl --user start calix-daemon.service`.\n",
                    .{socket_path},
                );
            },
            error.UnsupportedVersion => {
                try context.stderr.print(
                    "releases: the daemon on {s} speaks another protocol version than this client.\n",
                    .{socket_path},
                );
            },
            error.MalformedAnswer => {
                try context.stderr.print(
                    "releases: the daemon on {s} answered something this client cannot read.\n",
                    .{socket_path},
                );
            },
            error.SocketPathTooLong => {
                try context.stderr.print(
                    "releases: the daemon socket path is too long for a Unix socket: {s}\n",
                    .{socket_path},
                );
            },
        }
        return 1;
    };
    defer answer.deinit(context.allocator);

    return try emit(context, answer);
}

/// Prints the daemon's answer, or names why there is none.
fn emit(context: *runtime.RuntimeContext, answer: client.Answer) !u8 {
    if (!answer.ok) {
        if (answer.message) |message| {
            try context.stderr.print("releases: {s}\n", .{message});
        } else {
            try context.stderr.print(
                "releases: the daemon refused the request ({s})\n",
                .{answer.code orelse "failed"},
            );
        }
        return 1;
    }

    try context.stdout.writeAll(answer.data orelse "[]");
    try context.stdout.writeAll("\n");
    return 0;
}

fn documentPath(context: *runtime.RuntimeContext) ![]const u8 {
    return std.fs.path.join(
        context.allocator,
        &.{ try xdg.stateDirectory(context), tracked.file_name },
    );
}

fn optionEnabled(invocation: *const parser.Invocation, name: []const u8) bool {
    for (invocation.options) |option| {
        if (!std.mem.eql(u8, option.name, name)) continue;
        return option.value == null or !std.ascii.eqlIgnoreCase(option.value.?, "false");
    }
    return false;
}

const testing = std.testing;

fn commandOf(path: []const u8, name: []const u8) spec.Command {
    return .{
        .path = path,
        .parentPath = "calix",
        .name = name,
        .description = null,
        .hidden = false,
        .isBranch = false,
        .hasAction = true,
        .aliases = &.{},
        .arguments = &.{},
        .options = &.{},
    };
}

test "the releases command claims only its own path" {
    const releases_command = commandOf(command_path, "releases");
    const inventory_command = commandOf("calix inventory standard", "inventory");

    const releases_invocation: parser.Invocation = .{
        .command = &releases_command,
        .arguments = &.{},
        .positionals = &.{},
        .options = &.{},
        .globals = .{},
    };
    const inventory_invocation: parser.Invocation = .{
        .command = &inventory_command,
        .arguments = &.{},
        .positionals = &.{},
        .options = &.{},
        .globals = .{},
    };

    try testing.expect(claims(&releases_invocation));
    try testing.expect(!claims(&inventory_invocation));
}

/// A context whose state directory is `state`, so a test can put a document
/// where the command looks for one.
///
/// Set up in place rather than returned by value: `context.environment`
/// points at the fixture's own map, and a copy would leave it pointing at a
/// field of something that no longer exists.
const Fixture = struct {
    arena: std.heap.ArenaAllocator = undefined,
    environment: std.process.Environ.Map = undefined,
    stdout: std.Io.Writer.Allocating = undefined,
    stderr: std.Io.Writer.Allocating = undefined,
    context: runtime.RuntimeContext = undefined,

    fn setup(self: *Fixture, state: []const u8) !void {
        self.arena = std.heap.ArenaAllocator.init(testing.allocator);
        self.environment = std.process.Environ.Map.init(self.arena.allocator());
        self.stdout = std.Io.Writer.Allocating.init(testing.allocator);
        self.stderr = std.Io.Writer.Allocating.init(testing.allocator);
        try self.environment.put("XDG_STATE_HOME", state);
        self.context = .{
            .allocator = self.arena.allocator(),
            .io = testing.io,
            .stdout = &self.stdout.writer,
            .stderr = &self.stderr.writer,
            .environment = &self.environment,
        };
    }

    fn deinit(self: *Fixture) void {
        self.stdout.deinit();
        self.stderr.deinit();
        self.arena.deinit();
    }

    fn writeDocument(self: *Fixture, body: []const u8) !void {
        try std.Io.Dir.cwd().createDirPath(testing.io, try xdg.stateDirectory(&self.context));
        const path = try documentPath(&self.context);
        var file = try std.Io.Dir.cwd().createFile(testing.io, path, .{});
        defer file.close(testing.io);
        try file.writeStreamingAll(testing.io, body);
    }
};

test "a missing document is an empty set, not an error" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", testing.allocator);
    defer testing.allocator.free(root);

    var fixture: Fixture = undefined;
    try fixture.setup(root);
    defer fixture.deinit();

    try testing.expectEqual(@as(u8, 0), try printCommitted(&fixture.context));
    try testing.expectEqualStrings("{}\n", fixture.stdout.written());
    try testing.expectEqualStrings("", fixture.stderr.written());
}

test "the committed document is printed as the writer left it" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", testing.allocator);
    defer testing.allocator.free(root);

    var fixture: Fixture = undefined;
    try fixture.setup(root);
    defer fixture.deinit();

    const document = "{\n  \"artcraft-launcher\": {\n    \"installed_version\": \"0.8.1\"\n  }\n}\n";
    try fixture.writeDocument(document);

    try testing.expectEqual(@as(u8, 0), try printCommitted(&fixture.context));
    try testing.expectEqualStrings(document, fixture.stdout.written());
}

test "a malformed document is refused by name, not printed" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", testing.allocator);
    defer testing.allocator.free(root);

    var fixture: Fixture = undefined;
    try fixture.setup(root);
    defer fixture.deinit();

    try fixture.writeDocument("{ this is not json");

    try testing.expectEqual(@as(u8, 1), try printCommitted(&fixture.context));
    try testing.expectEqualStrings("", fixture.stdout.written());
    try testing.expect(std.mem.indexOf(u8, fixture.stderr.written(), "is not JSON") != null);
}

test "an answer's data is printed, and a refusal is named" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", testing.allocator);
    defer testing.allocator.free(root);

    var fixture: Fixture = undefined;
    try fixture.setup(root);
    defer fixture.deinit();

    try testing.expectEqual(@as(u8, 0), try emit(&fixture.context, .{ .ok = true, .data = "[]" }));
    try testing.expectEqualStrings("[]\n", fixture.stdout.written());

    try testing.expectEqual(@as(u8, 1), try emit(&fixture.context, .{
        .ok = false,
        .code = "failed",
        .message = "the forge did not answer",
    }));
    try testing.expect(std.mem.indexOf(u8, fixture.stderr.written(), "the forge did not answer") != null);
}

test "a refusal without a sentence still names its code" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", testing.allocator);
    defer testing.allocator.free(root);

    var fixture: Fixture = undefined;
    try fixture.setup(root);
    defer fixture.deinit();

    try testing.expectEqual(@as(u8, 1), try emit(&fixture.context, .{
        .ok = false,
        .code = "unknown-action",
    }));
    try testing.expect(std.mem.indexOf(u8, fixture.stderr.written(), "unknown-action") != null);
}

test "a refresh with no daemon says so and names the way to start it" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const root = try temporary.dir.realPathFileAlloc(testing.io, ".", testing.allocator);
    defer testing.allocator.free(root);

    var fixture: Fixture = undefined;
    try fixture.setup(root);
    defer fixture.deinit();
    try fixture.environment.put("XDG_RUNTIME_DIR", root);

    try testing.expectEqual(@as(u8, 1), try refresh(&fixture.context));
    try testing.expectEqualStrings("", fixture.stdout.written());
    try testing.expect(std.mem.indexOf(u8, fixture.stderr.written(), "systemctl --user start calix-daemon.service") != null);
}
