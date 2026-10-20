//! The release registry: what each update source last offered, and what the
//! user has already been told about.
//!
//! This is the one piece of genuinely persistent state in the update path.
//! The package inventory is derived on every read and never stored, because
//! it can always be reconstructed from the system. "I already told the user
//! about 2.1.0" cannot be reconstructed from anything, so it lives here.
//!
//! The file belongs to the daemon's state directory. A standalone CLI that
//! needs to write it either talks to the running daemon or refuses: two
//! writers on one file means one of them silently loses an update.
//!
//! Two calls, not one, because they can fail independently:
//!
//!   * `record` answers "is there something the user has not been told
//!     about?" and never marks anything as told.
//!   * `markNotified` is called only after the notification actually
//!     happened. A notify that fails leaves the answer to `record`
//!     unchanged, so the user is told next time instead of never.

const std = @import("std");
const release_version = @import("../shared/release_version.zig");
const json = @import("../shared/json.zig");

pub const Error = error{
    /// The registry exists but could not be read.
    Unreadable,
    /// The registry is not the shape this module writes.
    Malformed,
    /// The registry was written by a newer Calix than this one.
    UnsupportedVersion,
    /// The registry could not be written.
    Unwritable,
    /// An allocation failed.
    OutOfMemory,
};

/// The version of the registry format.
pub const format_version: u32 = 1;

/// What a probe of one update source returned.
pub const Probe = union(enum) {
    /// The source answered, offering this version. A source that answers
    /// without offering a version reports null rather than failing.
    seen: ?[]const u8,
    /// The source did not answer, for this reason.
    failed: []const u8,
};

/// One tracked artefact.
///
/// `name` is the key. The registry tracks release-managed artefacts only, so
/// an AppImage named `ripgrep` and a Cargo crate named `ripgrep` never
/// collide: only the AppImage has a release source.
pub const Entry = struct {
    name: []const u8,
    /// The update source kind: `github`, `gitlab`, `gitea`, `codeberg`,
    /// `forgejo` or `static_url`.
    source: []const u8 = "",
    /// The newest version the source offered at the last successful probe.
    last_seen: ?[]const u8 = null,
    /// The newest version the user has been told about.
    notified: ?[]const u8 = null,
    /// When the source was last probed successfully, in Unix seconds.
    last_checked: ?i64 = null,
    /// Why the last probe failed, when it failed.
    last_error: ?[]const u8 = null,
    /// How many consecutive probes have failed.
    consecutive_failures: u32 = 0,

    /// Whether the last seen version is one the user has not been told
    /// about.
    ///
    /// The rule lives here rather than in each caller, so the daemon that
    /// reports news and the registry that records it cannot disagree about
    /// what counts as news.
    pub fn hasNews(self: Entry) bool {
        return isNews(self.last_seen, self.notified);
    }

    pub fn deinit(self: *Entry, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.source);
        if (self.last_seen) |value| allocator.free(value);
        if (self.notified) |value| allocator.free(value);
        if (self.last_error) |value| allocator.free(value);
        self.* = undefined;
    }
};

/// The answer to "is there something the user has not been told about?".
pub const Outcome = struct {
    /// The version the source offers now, borrowed from the registry. Valid
    /// until the registry is next mutated.
    seen: ?[]const u8,
    /// Whether the source answered.
    answered: bool,
    /// Whether the user has not been told about `seen`.
    ///
    /// A version older than the one already notified is not news: a forge
    /// that flips its "latest" back to a prerelease must not send a user on
    /// a stable release a notification telling them to downgrade.
    is_news: bool,
};

pub const Registry = struct {
    allocator: std.mem.Allocator,
    entries: std.ArrayList(Entry) = .empty,

    pub fn init(allocator: std.mem.Allocator) Registry {
        return .{ .allocator = allocator };
    }

    pub fn deinit(self: *Registry) void {
        for (self.entries.items) |*entry| entry.deinit(self.allocator);
        self.entries.deinit(self.allocator);
        self.* = undefined;
    }

    /// Reads the registry.
    ///
    /// A missing file is an empty registry, not an error: it is what the
    /// first run sees, and what a user who never enabled release tracking
    /// keeps seeing.
    pub fn load(allocator: std.mem.Allocator, io: std.Io, path: []const u8) Error!Registry {
        const contents = readFile(allocator, io, path) catch |err| switch (err) {
            error.FileNotFound => return Registry.init(allocator),
            error.OutOfMemory => return error.OutOfMemory,
            else => return error.Unreadable,
        };
        defer allocator.free(contents);

        return parse(allocator, contents);
    }

    /// Writes the registry, atomically.
    ///
    /// A crash while writing must leave the previous registry intact, not a
    /// half-written file that loses every "already notified" decision. The
    /// bytes go to a sibling, are flushed, and are renamed into place.
    pub fn save(self: *Registry, io: std.Io, path: []const u8) Error!void {
        const bytes = std.json.Stringify.valueAlloc(
            self.allocator,
            File{ .version = format_version, .entries = self.entries.items },
            json.omit_null_optionals,
        ) catch return error.OutOfMemory;
        defer self.allocator.free(bytes);

        writeAtomically(self.allocator, io, path, bytes) catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return error.Unwritable,
        };
    }

    /// The entry for a name, if tracked.
    pub fn find(self: *Registry, name: []const u8) ?*Entry {
        for (self.entries.items) |*entry| {
            if (std.mem.eql(u8, entry.name, name)) return entry;
        }
        return null;
    }

    /// Records the outcome of a probe and answers whether it is news.
    ///
    /// Nothing is marked as notified here: the notification has not
    /// happened yet.
    pub fn record(
        self: *Registry,
        name: []const u8,
        source: []const u8,
        probe: Probe,
        now: i64,
    ) Error!Outcome {
        const entry = try self.findOrCreate(name, source);
        // Any append above invalidates pointers into the list, so resolve
        // the entry once the list has stopped growing.
        try self.setSource(entry, source);

        switch (probe) {
            .failed => |reason| {
                try self.replaceOwned(&entry.last_error, reason);
                entry.consecutive_failures +|= 1;
                return .{ .seen = entry.last_seen, .answered = false, .is_news = false };
            },
            .seen => |version| {
                if (entry.last_error) |value| {
                    self.allocator.free(value);
                    entry.last_error = null;
                }
                entry.consecutive_failures = 0;
                entry.last_checked = now;
                if (version) |text| {
                    try self.replaceOwned(&entry.last_seen, text);
                }
                return .{
                    .seen = entry.last_seen,
                    .answered = true,
                    .is_news = isNews(entry.last_seen, entry.notified),
                };
            },
        }
    }

    /// Marks a version as told.
    ///
    /// Called after the notification succeeded, never before.
    pub fn markNotified(self: *Registry, name: []const u8, version: []const u8) Error!void {
        const entry = self.find(name) orelse return;
        try self.replaceOwned(&entry.notified, version);
    }

    fn findOrCreate(self: *Registry, name: []const u8, source: []const u8) Error!*Entry {
        if (self.find(name)) |entry| return entry;

        const owned_name = try self.allocator.dupe(u8, name);
        errdefer self.allocator.free(owned_name);
        const owned_source = try self.allocator.dupe(u8, source);
        errdefer self.allocator.free(owned_source);

        try self.entries.append(self.allocator, .{
            .name = owned_name,
            .source = owned_source,
        });
        return &self.entries.items[self.entries.items.len - 1];
    }

    fn setSource(self: *Registry, entry: *Entry, source: []const u8) Error!void {
        if (std.mem.eql(u8, entry.source, source)) return;
        const owned = try self.allocator.dupe(u8, source);
        self.allocator.free(entry.source);
        entry.source = owned;
    }

    fn replaceOwned(self: *Registry, slot: *?[]const u8, value: []const u8) Error!void {
        const owned = try self.allocator.dupe(u8, value);
        if (slot.*) |previous| self.allocator.free(previous);
        slot.* = owned;
    }
};

/// Whether a version is news relative to what the user was told.
fn isNews(seen: ?[]const u8, notified: ?[]const u8) bool {
    const current = seen orelse return false;
    const told = notified orelse return true;
    if (std.mem.eql(u8, current, told)) return false;
    // A forge that flips its "latest" back to a prerelease is not offering
    // the user an upgrade. `isNewer` returns null for tags it cannot order,
    // and those are reported: a change the user cannot be told about is
    // worse than a notification they can dismiss.
    return release_version.isNewer(current, told) orelse true;
}

/// The on-disk shape.
const File = struct {
    version: u32 = format_version,
    entries: []const Entry = &.{},
};

fn parse(allocator: std.mem.Allocator, contents: []const u8) Error!Registry {
    const parsed = std.json.parseFromSlice(File, allocator, contents, .{
        .ignore_unknown_fields = true,
    }) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.Malformed,
    };
    defer parsed.deinit();

    if (parsed.value.version != format_version) return error.UnsupportedVersion;

    var registry = Registry.init(allocator);
    errdefer registry.deinit();

    try registry.entries.ensureTotalCapacity(allocator, parsed.value.entries.len);
    for (parsed.value.entries) |stored| {
        const owned_name = try allocator.dupe(u8, stored.name);
        errdefer allocator.free(owned_name);
        const owned_source = try allocator.dupe(u8, stored.source);
        errdefer allocator.free(owned_source);
        // These are optionals: an absent field must not be handed to free.
        const owned_seen = try dupOptional(allocator, stored.last_seen);
        errdefer if (owned_seen) |value| allocator.free(value);
        const owned_notified = try dupOptional(allocator, stored.notified);
        errdefer if (owned_notified) |value| allocator.free(value);
        const owned_error = try dupOptional(allocator, stored.last_error);
        errdefer if (owned_error) |value| allocator.free(value);

        registry.entries.appendAssumeCapacity(.{
            .name = owned_name,
            .source = owned_source,
            .last_seen = owned_seen,
            .notified = owned_notified,
            .last_checked = stored.last_checked,
            .last_error = owned_error,
            .consecutive_failures = stored.consecutive_failures,
        });
    }

    return registry;
}

fn dupOptional(allocator: std.mem.Allocator, value: ?[]const u8) Error!?[]const u8 {
    const text = value orelse return null;
    return try allocator.dupe(u8, text);
}

fn readFile(allocator: std.mem.Allocator, io: std.Io, path: []const u8) ![]u8 {
    var file = try std.Io.Dir.cwd().openFile(io, path, .{});
    defer file.close(io);

    var buffer: [8192]u8 = undefined;
    var reader = file.reader(io, &buffer);
    return try reader.interface.allocRemaining(allocator, .unlimited);
}

fn writeAtomically(
    allocator: std.mem.Allocator,
    io: std.Io,
    path: []const u8,
    bytes: []const u8,
) !void {
    if (std.fs.path.dirname(path)) |directory| {
        var handle = try std.Io.Dir.cwd().createDirPathOpen(io, directory, .{});
        handle.close(io);
    }

    const nonce = std.Io.Timestamp.now(io, .awake).nanoseconds;
    const staging_path = try std.fmt.allocPrint(allocator, "{s}.tmp-{d}", .{ path, nonce });
    defer allocator.free(staging_path);
    defer std.Io.Dir.cwd().deleteFile(io, staging_path) catch {};

    {
        var file = try std.Io.Dir.cwd().createFile(io, staging_path, .{ .exclusive = true });
        defer file.close(io);
        var write_buffer: [4096]u8 = undefined;
        var writer = file.writer(io, &write_buffer);
        try writer.interface.writeAll(bytes);
        try writer.interface.flush();
        try file.sync(io);
    }

    try std.Io.Dir.rename(.cwd(), staging_path, .cwd(), path, io);
}

const testing = std.testing;

/// The absolute path of a file inside a test's temporary directory.
///
/// The file need not exist: several tests are about absent registries and
/// directories that save has to create.
fn scratchPath(temporary: *testing.TmpDir, name: []const u8) ![]u8 {
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    return try std.fs.path.join(testing.allocator, &.{ buffer[0..length], name });
}

test "load: a missing registry is empty, not an error" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var registry = try Registry.load(testing.allocator, testing.io, path);
    defer registry.deinit();
    try testing.expectEqual(@as(usize, 0), registry.entries.items.len);
}

test "save and load: a registry round-trips" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var registry = Registry.init(testing.allocator);
    defer registry.deinit();
    _ = try registry.record("CADCraft", "github", .{ .seen = "0.9.0" }, 1_700_000_000);
    try registry.markNotified("CADCraft", "0.8.0");

    try registry.save(testing.io, path);

    var reloaded = try Registry.load(testing.allocator, testing.io, path);
    defer reloaded.deinit();

    try testing.expectEqual(@as(usize, 1), reloaded.entries.items.len);
    const entry = reloaded.find("CADCraft").?;
    try testing.expectEqualStrings("github", entry.source);
    try testing.expectEqualStrings("0.9.0", entry.last_seen.?);
    try testing.expectEqualStrings("0.8.0", entry.notified.?);
    try testing.expectEqual(@as(i64, 1_700_000_000), entry.last_checked.?);
    try testing.expectEqual(@as(u32, 0), entry.consecutive_failures);
}

test "save: a null optional is omitted, not written as null" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var registry = Registry.init(testing.allocator);
    defer registry.deinit();
    _ = try registry.record("Fresh", "github", .{ .seen = "1.0.0" }, 1_700_000_000);
    try registry.save(testing.io, path);

    const contents = try readFile(testing.allocator, testing.io, path);
    defer testing.allocator.free(contents);
    try testing.expect(std.mem.indexOf(u8, contents, "\"notified\"") == null);
    try testing.expect(std.mem.indexOf(u8, contents, "null") == null);
    try testing.expect(std.mem.indexOf(u8, contents, "\"last_seen\":\"1.0.0\"") != null);
}

test "record: a first sighting is news, and a repeat is not" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    const first = try registry.record("App", "github", .{ .seen = "1.0.0" }, 100);
    try testing.expect(first.answered);
    try testing.expect(first.is_news);

    try registry.markNotified("App", "1.0.0");

    const repeated = try registry.record("App", "github", .{ .seen = "1.0.0" }, 200);
    try testing.expect(!repeated.is_news);

    const newer = try registry.record("App", "github", .{ .seen = "1.1.0" }, 300);
    try testing.expect(newer.is_news);
}

test "record: a forge rolling back to a prerelease is not an upgrade" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    _ = try registry.record("App", "github", .{ .seen = "1.7.4" }, 100);
    try registry.markNotified("App", "1.7.4");

    // The forge's "latest" flips to its own prerelease of the same release.
    // Telling a user on 1.7.4 to "update" to 1.7.4-rc1 is the bug the
    // release-tag ordering exists to prevent.
    const outcome = try registry.record("App", "github", .{ .seen = "1.7.4-rc1" }, 200);
    try testing.expect(!outcome.is_news);

    // A genuine rollback to an older release is not news either.
    const rolled_back = try registry.record("App", "github", .{ .seen = "1.6.0" }, 300);
    try testing.expect(!rolled_back.is_news);
}

test "record: a version that cannot be ordered is still reported" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    _ = try registry.record("App", "github", .{ .seen = "2024.10" }, 100);
    try registry.markNotified("App", "2024.10");

    const outcome = try registry.record("App", "github", .{ .seen = "nightly" }, 200);
    try testing.expect(outcome.is_news);
}

test "record: a failed probe keeps the last seen version and counts the failure" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    _ = try registry.record("App", "github", .{ .seen = "2.0.0" }, 100);
    const failed = try registry.record("App", "github", .{ .failed = "HTTP 503" }, 200);

    try testing.expect(!failed.answered);
    try testing.expect(!failed.is_news);
    try testing.expectEqualStrings("2.0.0", failed.seen.?);

    const entry = registry.find("App").?;
    try testing.expectEqualStrings("HTTP 503", entry.last_error.?);
    try testing.expectEqual(@as(u32, 1), entry.consecutive_failures);
    // A failure is not a check: the timestamp still says when the source
    // last answered.
    try testing.expectEqual(@as(i64, 100), entry.last_checked.?);

    _ = try registry.record("App", "github", .{ .failed = "HTTP 503" }, 300);
    try testing.expectEqual(@as(u32, 2), registry.find("App").?.consecutive_failures);
}

test "record: the first success after failures clears the error" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    _ = try registry.record("App", "github", .{ .failed = "timeout" }, 100);
    try testing.expectEqual(@as(u32, 1), registry.find("App").?.consecutive_failures);

    _ = try registry.record("App", "github", .{ .seen = "3.0.0" }, 200);
    const entry = registry.find("App").?;
    try testing.expect(entry.last_error == null);
    try testing.expectEqual(@as(u32, 0), entry.consecutive_failures);
    try testing.expectEqual(@as(i64, 200), entry.last_checked.?);
}

test "record: a source that answers without a version is not news" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    const outcome = try registry.record("App", "static_url", .{ .seen = null }, 100);
    try testing.expect(outcome.answered);
    try testing.expect(!outcome.is_news);
    try testing.expect(outcome.seen == null);
}

test "record: an empty probe with a stored version keeps the stored one" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    _ = try registry.record("App", "github", .{ .seen = "1.0.0" }, 100);
    const outcome = try registry.record("App", "github", .{ .seen = null }, 200);
    try testing.expectEqualStrings("1.0.0", outcome.seen.?);
    try testing.expect(outcome.is_news);
}

test "markNotified: an untracked name is a no-op, not a crash" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();
    try registry.markNotified("Never seen", "1.0.0");
    try testing.expectEqual(@as(usize, 0), registry.entries.items.len);
}

test "record: a source change is recorded, not ignored" {
    var registry = Registry.init(testing.allocator);
    defer registry.deinit();

    _ = try registry.record("App", "github", .{ .seen = "1.0.0" }, 100);
    _ = try registry.record("App", "gitlab", .{ .seen = "1.0.0" }, 200);
    try testing.expectEqualStrings("gitlab", registry.find("App").?.source);
    // The name is the key: a source change must not create a second entry.
    try testing.expectEqual(@as(usize, 1), registry.entries.items.len);
}

test "load: a registry of an unknown version is refused" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data = "{\"version\":99,\"entries\":[]}",
    });

    try testing.expectError(error.UnsupportedVersion, Registry.load(testing.allocator, testing.io, path));
}

test "load: a malformed registry is refused rather than silently emptied" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data = "not json",
    });

    try testing.expectError(error.Malformed, Registry.load(testing.allocator, testing.io, path));
}

test "load: an unknown field does not cost the user their registry" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"version":1,"entries":[{"name":"App","source":"github","last_seen":"1.0.0","future_field":true}]}
        ,
    });

    var registry = try Registry.load(testing.allocator, testing.io, path);
    defer registry.deinit();
    try testing.expectEqualStrings("1.0.0", registry.find("App").?.last_seen.?);
}

test "save: the parent directory is created" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "state/releases.json");
    defer testing.allocator.free(path);

    var registry = Registry.init(testing.allocator);
    defer registry.deinit();
    _ = try registry.record("App", "github", .{ .seen = "1.0.0" }, 100);

    try registry.save(testing.io, path);

    var reloaded = try Registry.load(testing.allocator, testing.io, path);
    defer reloaded.deinit();
    try testing.expectEqual(@as(usize, 1), reloaded.entries.items.len);
}

test "save: no staging file is left behind" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var registry = Registry.init(testing.allocator);
    defer registry.deinit();
    try registry.save(testing.io, path);

    var directory = try temporary.dir.openDir(testing.io, ".", .{ .iterate = true });
    defer directory.close(testing.io);
    var names = directory.iterate();
    while (try names.next(testing.io)) |entry| {
        try testing.expect(std.mem.indexOf(u8, entry.name, ".tmp-") == null);
    }
}
