//! Test support: an environment isolated from the user's real XDG directories.
//!
//! The AppImage tests install AppImages and write desktop entries. Run with
//! the real process environment, they wrote into the user's own
//! `~/.local/share/applications` and left entries behind that pointed at
//! build temporaries — litter that outlived the test that created it.
//!
//! Every test that touches the filesystem must use `isolatedEnviron()`
//! instead of `std.testing.environ`.

const std = @import("std");

var cached: ?std.process.Environ = null;

/// The environment every AppImage test must use.
///
/// It is created once for the test binary and rooted in a temporary
/// directory, so no test can reach the user's real `~/.local/share`,
/// `~/.cache` or `~/.config`.
///
/// The block is allocated from the page allocator on purpose: it lives for
/// the whole test binary, and the testing allocator would otherwise report
/// it as a leak at the end of the run.
pub fn isolatedEnviron() std.process.Environ {
    if (cached) |environ| return environ;

    var dir = std.testing.tmpDir(.{});
    errdefer dir.cleanup();

    var path_buf: [std.Io.Dir.max_path_bytes]u8 = undefined;
    const len = dir.dir.realPath(std.testing.io, &path_buf) catch @panic("cannot resolve the test root");
    const root = path_buf[0..len];

    const data_home = std.fs.path.join(std.heap.page_allocator, &.{ root, "data" }) catch @panic("cannot build the test data home");
    std.Io.Dir.cwd().createDirPath(std.testing.io, data_home) catch @panic("cannot create the test data home");

    var map = std.process.Environ.Map.init(std.heap.page_allocator);
    defer map.deinit();
    map.put("HOME", root) catch @panic("cannot set HOME");
    map.put("XDG_DATA_HOME", data_home) catch @panic("cannot set XDG_DATA_HOME");
    map.put("XDG_CACHE_HOME", root) catch @panic("cannot set XDG_CACHE_HOME");

    // createPosixBlock allocates its own strings, so the block stays valid
    // after the map is released.
    const block = map.createPosixBlock(std.heap.page_allocator, .{}) catch @panic("cannot build the test environment");
    cached = .{ .block = block };
    return cached.?;
}

test "isolatedEnviron: the environment points inside a temporary root" {
    const environ = isolatedEnviron();
    const home = std.process.Environ.getPosix(environ, "HOME").?;
    try std.testing.expect(std.mem.indexOf(u8, std.process.Environ.getPosix(environ, "XDG_DATA_HOME").?, home) != null);
    try std.testing.expect(std.mem.indexOf(u8, std.process.Environ.getPosix(environ, "XDG_CACHE_HOME").?, home) != null);
    // It must not be the user's own home.
    try std.testing.expect(std.mem.indexOf(u8, home, "/tmp/") != null or std.mem.indexOf(u8, home, "/var/tmp/") != null);
}
