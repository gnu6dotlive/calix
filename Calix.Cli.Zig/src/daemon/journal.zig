//! The daemon's operational journal.
//!
//! One JSON object per line, one file per day, in the daemon's own state
//! directory. It stays out of the repository's `evidence/` tree on purpose:
//! that tree records what was proven in a run, and this records what the
//! daemon did. Mixing them would make the evidence tree grow with every
//! request and make a run's record impossible to freeze.
//!
//! A journal write never fails a request. A daemon that refuses to serve
//! because it cannot append a log line is worse than a daemon with a gap in
//! its log, so the failure is reported and the request continues.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const json_options = Zigalpm.shared.json.omit_null_optionals;

/// One journal line.
pub const Event = struct {
    /// Unix seconds.
    at: i64,
    /// The action that was served, or `daemon` for a lifecycle event.
    action: []const u8,
    ok: bool,
    /// A short human-readable note.
    message: ?[]const u8 = null,
};

pub const Journal = struct {
    allocator: std.mem.Allocator,
    io: std.Io,
    directory: []const u8,

    pub fn init(allocator: std.mem.Allocator, io: std.Io, directory: []const u8) Journal {
        return .{ .allocator = allocator, .io = io, .directory = directory };
    }

    /// Appends an event, best effort.
    pub fn append(self: Journal, event: Event) void {
        self.appendOrReport(event) catch |err| {
            std.log.warn("calix-daemon: could not append to the journal: {s}", .{@errorName(err)});
        };
    }

    fn appendOrReport(self: Journal, event: Event) !void {
        var handle = try std.Io.Dir.cwd().createDirPathOpen(self.io, self.directory, .{
            .permissions = .fromMode(0o700),
        });
        handle.close(self.io);

        const name = try dayFileName(self.allocator, event.at);
        defer self.allocator.free(name);
        const path = try std.fs.path.join(self.allocator, &.{ self.directory, name });
        defer self.allocator.free(path);

        const line = try std.json.Stringify.valueAlloc(self.allocator, event, json_options);
        defer self.allocator.free(line);

        var file = try std.Io.Dir.cwd().createFile(self.io, path, .{ .truncate = false });
        defer file.close(self.io);
        // Append at the end rather than seeking: a crash mid-write costs at
        // most the last line, never the file.
        const end = (try file.stat(self.io)).size;
        try file.writePositionalAll(self.io, line, end);
        try file.writePositionalAll(self.io, "\n", end + line.len);
    }
};

/// `daemon-YYYY-MM-DD.jsonl` for a Unix timestamp.
fn dayFileName(allocator: std.mem.Allocator, timestamp: i64) ![]u8 {
    const seconds: u64 = @intCast(@max(0, timestamp));
    const epoch = std.time.epoch.EpochSeconds{ .secs = seconds };
    const year_day = epoch.getEpochDay().calculateYearDay();
    const month_day = year_day.calculateMonthDay();
    return std.fmt.allocPrint(allocator, "daemon-{d:0>4}-{d:0>2}-{d:0>2}.jsonl", .{
        year_day.year,
        // `Month.jan` is 1, and its `numeric()` is the calendar value.
        // `@intFromEnum` would be the same number; adding one to it, which
        // is the natural mistake, shifts every journal a month.
        month_day.month.numeric(),
        month_day.day_index + 1,
    });
}

const testing = std.testing;

test "dayFileName: a timestamp becomes the day it fell on" {
    // 2026-10-20T00:00:00Z
    const name = try dayFileName(testing.allocator, 1_792_454_400);
    defer testing.allocator.free(name);
    try testing.expectEqualStrings("daemon-2026-10-20.jsonl", name);
}

test "dayFileName: the epoch itself is 1970-01-01" {
    const name = try dayFileName(testing.allocator, 0);
    defer testing.allocator.free(name);
    try testing.expectEqualStrings("daemon-1970-01-01.jsonl", name);
}

test "Journal: an event is one JSON line, appended" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    const directory = try testing.allocator.dupe(u8, buffer[0..length]);
    defer testing.allocator.free(directory);

    const journal = Journal.init(testing.allocator, testing.io, directory);
    journal.append(.{ .at = 1_792_454_400, .action = "inventory", .ok = true, .message = "served" });
    journal.append(.{ .at = 1_792_454_400, .action = "releases", .ok = false, .message = "probe failed" });

    const path = try std.fs.path.join(testing.allocator, &.{ directory, "daemon-2026-10-20.jsonl" });
    defer testing.allocator.free(path);
    const contents = try std.Io.Dir.cwd().readFileAlloc(
        testing.io,
        path,
        testing.allocator,
        .limited(64 * 1024),
    );
    defer testing.allocator.free(contents);

    var lines = std.mem.splitScalar(u8, std.mem.trimEnd(u8, contents, "\n"), '\n');
    const first = lines.next().?;
    const second = lines.next().?;
    try testing.expect(lines.next() == null);

    try testing.expect(std.mem.indexOf(u8, first, "\"action\":\"inventory\"") != null);
    try testing.expect(std.mem.indexOf(u8, first, "\"ok\":true") != null);
    try testing.expect(std.mem.indexOf(u8, second, "\"action\":\"releases\"") != null);
    try testing.expect(std.mem.indexOf(u8, second, "\"ok\":false") != null);
}

test "Journal: an unwritable directory is reported, not fatal" {
    var temporary = testing.tmpDir(.{});

    // A regular file where the directory should be: creating the journal
    // directory under it fails with `NotDir`, immediately and for the
    // reason under test. (`/proc` also fails, but it can block, and a test
    // that hangs is worse than a test that fails.)
    try temporary.dir.writeFile(testing.io, .{ .sub_path = "blocked", .data = "" });
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    const blocked = try std.fs.path.join(
        testing.allocator,
        &.{ buffer[0..length], "blocked", "journal" },
    );
    defer testing.allocator.free(blocked);

    const journal = Journal.init(testing.allocator, testing.io, blocked);
    // No error escapes: a journal that cannot be written must not take the
    // daemon down with it.
    journal.append(.{ .at = 0, .action = "daemon", .ok = true });
    temporary.cleanup();
}
