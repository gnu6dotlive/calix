//! Where the daemon listens.
//!
//! One definition, used by the daemon that binds the socket and by the CLI
//! that asks through it. Two copies of this path would drift the day one of
//! them learned a new fallback, and the failure would look like "the daemon
//! is not running" on a machine where it is.

const std = @import("std");

const runtime = @import("../runtime/context.zig");
const xdg = @import("../runtime/xdg.zig");

/// `$XDG_RUNTIME_DIR/calix/daemon.sock`, or `/tmp/calix-<uid>/daemon.sock`
/// when the session has no runtime directory.
pub fn socketPath(
    allocator: std.mem.Allocator,
    context: *const runtime.RuntimeContext,
) ![]const u8 {
    if (xdg.getEnv(context, "XDG_RUNTIME_DIR")) |runtime_directory| {
        if (runtime_directory.len > 0) {
            return std.fs.path.join(allocator, &.{ runtime_directory, "calix", "daemon.sock" });
        }
    }

    const uid = std.os.linux.getuid();
    return std.fmt.allocPrint(allocator, "/tmp/calix-{d}/daemon.sock", .{uid});
}

test "the socket lives under the runtime directory when there is one" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();

    var environment = std.process.Environ.Map.init(arena.allocator());
    try environment.put("XDG_RUNTIME_DIR", "/run/user/1000");
    const context: runtime.RuntimeContext = .{
        .allocator = arena.allocator(),
        .io = std.testing.io,
        .stdout = undefined,
        .stderr = undefined,
        .environment = &environment,
    };

    try std.testing.expectEqualStrings(
        "/run/user/1000/calix/daemon.sock",
        try socketPath(arena.allocator(), &context),
    );
}

test "without a runtime directory the socket falls back to the uid" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();

    var environment = std.process.Environ.Map.init(arena.allocator());
    const context: runtime.RuntimeContext = .{
        .allocator = arena.allocator(),
        .io = std.testing.io,
        .stdout = undefined,
        .stderr = undefined,
        .environment = &environment,
    };

    const path = try socketPath(arena.allocator(), &context);
    try std.testing.expect(std.mem.startsWith(u8, path, "/tmp/calix-"));
    try std.testing.expect(std.mem.endsWith(u8, path, "/daemon.sock"));
}
