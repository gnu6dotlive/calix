//! `calix-daemon` — the process the frontend talks to.
//!
//! It is deliberately dull: parse two paths, build a runtime context, serve.
//! Everything that decides anything lives in `server.zig`, where it can be
//! tested without a socket.

const std = @import("std");
const Io = std.Io;

const Calix_Cli_Zig = @import("Calix_Cli_Zig");
const Zigalpm = @import("Zigalpm");

const runtime = Calix_Cli_Zig.runtime;
const xdg = Calix_Cli_Zig.xdg;
const server = Calix_Cli_Zig.daemon.server;

const usage =
    \\Usage: calix-daemon [options]
    \\
    \\Runs the Calix daemon: the only writer of Calix state, and the only
    \\process the frontend reads from.
    \\
    \\Options:
    \\  --socket <path>     Serve on this Unix socket
    \\  --state-dir <path>  Keep state in this directory
    \\  --version           Print the version
    \\  -h, --help          Print this help
    \\
    \\Defaults follow the XDG base directory specification:
    \\  socket  $XDG_RUNTIME_DIR/calix/daemon.sock
    \\  state   $XDG_STATE_HOME/calix
    \\
;

pub fn main(init: std.process.Init) !void {
    const arena = init.arena.allocator();
    const io = init.io;

    var stdout_buffer: [4096]u8 = undefined;
    var stdout_file_writer: Io.File.Writer = .init(.stdout(), io, &stdout_buffer);
    const stdout = &stdout_file_writer.interface;
    var stderr_buffer: [4096]u8 = undefined;
    var stderr_file_writer: Io.File.Writer = .init(.stderr(), io, &stderr_buffer);
    const stderr = &stderr_file_writer.interface;

    const arguments = try init.minimal.args.toSlice(arena);

    var socket_argument: ?[]const u8 = null;
    var state_argument: ?[]const u8 = null;
    var index: usize = 1;
    while (index < arguments.len) : (index += 1) {
        const argument = arguments[index];
        if (std.mem.eql(u8, argument, "-h") or std.mem.eql(u8, argument, "--help")) {
            try stdout.writeAll(usage);
            try stdout.flush();
            return;
        } else if (std.mem.eql(u8, argument, "--version")) {
            try stdout.print("calix-daemon {s}\n", .{@import("build_options").version});
            try stdout.flush();
            return;
        } else if (std.mem.eql(u8, argument, "--socket")) {
            index += 1;
            if (index >= arguments.len) return usageError(stderr, "--socket needs a path");
            socket_argument = arguments[index];
        } else if (std.mem.eql(u8, argument, "--state-dir")) {
            index += 1;
            if (index >= arguments.len) return usageError(stderr, "--state-dir needs a path");
            state_argument = arguments[index];
        } else {
            return usageError(stderr, argument);
        }
    }

    var context = runtime.RuntimeContext{
        .allocator = init.gpa,
        .io = io,
        .stdout = stdout,
        .stderr = stderr,
        .environment = init.environ_map,
        .environ = init.minimal.environ,
    };
    Zigalpm.HttpClient.setDefaultProxyEnvironment(init.environ_map);

    const socket_path = socket_argument orelse try defaultSocketPath(arena, &context);
    const state_directory = state_argument orelse try std.fs.path.join(
        arena,
        &.{ try xdg.stateHome(&context), "calix" },
    );

    var daemon = try server.Server.init(init.gpa, io, &context, .{
        .socket_path = socket_path,
        .state_directory = state_directory,
    });
    defer daemon.deinit();

    daemon.run() catch |err| switch (err) {
        error.AlreadyRunning => {
            // Not a failure worth a stack trace: the user has a daemon.
            try stderr.print(
                "calix-daemon: a daemon is already serving {s}\n",
                .{socket_path},
            );
            try stderr.flush();
            std.process.exit(1);
        },
        else => {
            try stderr.print("calix-daemon: {s}\n", .{@errorName(err)});
            try stderr.flush();
            std.process.exit(1);
        },
    };
}

/// The socket to serve on when the caller did not choose one.
///
/// `XDG_RUNTIME_DIR` is the right home: it is per-user, mode 0700, and
/// cleared when the session ends, so a socket left behind by a crash does
/// not outlive the session that produced it.
fn defaultSocketPath(allocator: std.mem.Allocator, context: *const runtime.RuntimeContext) ![]const u8 {
    if (xdg.getEnv(context, "XDG_RUNTIME_DIR")) |runtime_directory| {
        if (runtime_directory.len > 0) {
            return std.fs.path.join(allocator, &.{ runtime_directory, "calix", "daemon.sock" });
        }
    }

    const uid = std.os.linux.getuid();
    return std.fmt.allocPrint(allocator, "/tmp/calix-{d}/daemon.sock", .{uid});
}

fn usageError(stderr: *std.Io.Writer, detail: []const u8) !void {
    try stderr.print("calix-daemon: unexpected argument '{s}'\n\n{s}", .{ detail, usage });
    try stderr.flush();
    std.process.exit(2);
}
