//! The release document: what each update source last offered, and what the
//! user has already been told about.
//!
//! # One document, schema 2
//!
//! The document used to be two: the frozen Python engine's observations in
//! one file, and the daemon's notification memory in another. They described
//! the same packages and were confused for each other once already, so schema
//! 2 merges them: an observation carries the probe's answer and the
//! notification decision together, and the confusion is no longer
//! representable.
//!
//! This is the one piece of genuinely persistent state in the update path.
//! The package inventory is derived on every read and never stored, because
//! it can always be reconstructed from the system. "I already told the user
//! about 2.1.0" cannot be reconstructed from anything, so it lives here.
//!
//! The document belongs to the daemon's state directory, and the daemon is
//! its only writer. A standalone CLI that needs it written either talks to
//! the running daemon or refuses: two writers on one file means one of them
//! silently loses an update.
//!
//! # Two calls, not one
//!
//! They can fail independently:
//!
//!   * `record` answers "is there something the user has not been told
//!     about?" and never marks anything as told.
//!   * `markNotified` is called only after the notification actually
//!     happened. A notify that fails leaves the answer to `record`
//!     unchanged, so the user is told next time instead of never.
//!
//! # Reading an older document
//!
//! Schema 1 is the bare map of name to observation that the Python engine
//! wrote: no envelope, no version. It is migrated on read, because the
//! daemon is the next thing to write this path and a document that people
//! already have must not be thrown away. A document from a *newer* schema is
//! refused by a named error rather than read as if it were this one.

const std = @import("std");
const release_version = @import("../shared/release_version.zig");
const json = @import("../shared/json.zig");

pub const Error = error{
    /// The document exists but could not be read.
    Unreadable,
    /// The document is not the shape this module writes.
    Malformed,
    /// The document was written by a newer Calix than this one.
    UnsupportedVersion,
    /// The document could not be written.
    Unwritable,
    /// An allocation failed.
    OutOfMemory,
};

/// The version of the document format.
///
/// 1 — a bare map of name to observation (the frozen Python engine).
/// 2 — the envelope: `version`, `checked_at`, `last_error`, `entries`.
pub const version: u32 = 2;

/// The name of the document in the state directory.
pub const file_name = "releases.json";

/// What a probe of one update source returned.
pub const Probe = union(enum) {
    /// The source answered, offering this version. A source that answers
    /// without offering a version reports null rather than failing.
    seen: ?[]const u8,
    /// The source did not answer, for this reason.
    failed: []const u8,
};

/// What identifies the artefact being observed.
///
/// Carried onto the observation so one document says both what is watched
/// and what was seen: `installed_version` is the base's answer, `last_seen`
/// the source's.
pub const Subject = struct {
    name: []const u8,
    /// The update source kind: `github`, `static_url`, ...
    source: []const u8,
    installed_version: ?[]const u8 = null,
    repo: ?[]const u8 = null,
    provider: ?[]const u8 = null,
    canal: ?[]const u8 = null,
    pkgname: ?[]const u8 = null,
};

/// One tracked artefact.
///
/// `name` is the key. The document tracks release-managed artefacts only, so
/// an AppImage named `ripgrep` and a Cargo crate named `ripgrep` never
/// collide: only the AppImage has a release source.
pub const Observation = struct {
    name: []const u8,
    /// The update source kind: `github`, `gitlab`, `gitea`, `codeberg`,
    /// `forgejo` or `static_url`.
    source: []const u8 = "",
    /// The version the base says is installed, as of this observation.
    installed_version: ?[]const u8 = null,
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
    repo: ?[]const u8 = null,
    provider: ?[]const u8 = null,
    canal: ?[]const u8 = null,
    pkgname: ?[]const u8 = null,

    /// Whether the last seen version is one the user has not been told
    /// about.
    ///
    /// The rule lives here rather than in each caller, so the daemon that
    /// reports news and the document that records it cannot disagree about
    /// what counts as news.
    pub fn hasNews(self: Observation) bool {
        return isNews(self.last_seen, self.notified);
    }

    /// Whether the source is offering something newer than what is
    /// installed.
    ///
    /// A different question from `hasNews`: this one ignores what the user
    /// has been told. A version that cannot be ordered is not offered as an
    /// update — the same rule the Python engine applied, and the one that
    /// keeps a `nightly` tag from being announced as an upgrade.
    pub fn hasUpdate(self: Observation) bool {
        const installed = self.installed_version orelse return false;
        const seen = self.last_seen orelse return false;
        return release_version.isNewer(seen, installed) orelse false;
    }

    pub fn deinit(self: *Observation, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.source);
        for ([_]*?[]const u8{
            &self.installed_version,
            &self.last_seen,
            &self.notified,
            &self.last_error,
            &self.repo,
            &self.provider,
            &self.canal,
            &self.pkgname,
        }) |slot| {
            if (slot.*) |value| allocator.free(value);
        }
        self.* = undefined;
    }
};

/// The answer to "is there something the user has not been told about?".
pub const Outcome = struct {
    /// The version the source offers now, borrowed from the document. Valid
    /// until the document is next mutated.
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

pub const Document = struct {
    allocator: std.mem.Allocator,
    /// When this document was last committed, in Unix seconds.
    checked_at: ?i64 = null,
    /// Why the last refresh produced nothing, when it produced nothing.
    last_error: ?[]const u8 = null,
    entries: std.ArrayList(Observation) = .empty,

    pub fn init(allocator: std.mem.Allocator) Document {
        return .{ .allocator = allocator };
    }

    pub fn deinit(self: *Document) void {
        for (self.entries.items) |*entry| entry.deinit(self.allocator);
        self.entries.deinit(self.allocator);
        if (self.last_error) |value| self.allocator.free(value);
        self.* = undefined;
    }

    /// Reads the document.
    ///
    /// A missing file is an empty document, not an error: it is what the
    /// first run sees, and what a user who never enabled release tracking
    /// keeps seeing.
    pub fn load(allocator: std.mem.Allocator, io: std.Io, path: []const u8) Error!Document {
        const contents = readFile(allocator, io, path) catch |err| switch (err) {
            error.FileNotFound => return Document.init(allocator),
            error.OutOfMemory => return error.OutOfMemory,
            else => return error.Unreadable,
        };
        defer allocator.free(contents);

        return parse(allocator, contents);
    }

    /// Writes the document, atomically.
    ///
    /// A crash while writing must leave the previous document intact, not a
    /// half-written file that loses every "already notified" decision. The
    /// bytes go to a sibling, are flushed, and are renamed into place.
    ///
    /// Entries are sorted by name so that two documents holding the same set
    /// are the same bytes, whatever order the probes finished in.
    pub fn save(self: *Document, io: std.Io, path: []const u8) Error!void {
        std.mem.sort(Observation, self.entries.items, {}, byName);

        const bytes = try self.toJson(self.allocator, null);
        defer self.allocator.free(bytes);

        writeAtomically(self.allocator, io, path, bytes) catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return error.Unwritable,
        };
    }

    /// The document as the wire spells it, and as the file stores it.
    ///
    /// `refreshing` is the one field that is not part of the document: a
    /// request that started a probe says so, and a document on disk has no
    /// opinion about what the daemon is doing right now. It is omitted when
    /// absent, which is why one serializer serves both.
    pub fn toJson(
        self: *const Document,
        allocator: std.mem.Allocator,
        refreshing: ?bool,
    ) Error![]u8 {
        return std.json.Stringify.valueAlloc(
            allocator,
            Wire{
                .version = version,
                .checked_at = self.checked_at,
                .last_error = self.last_error,
                .entries = self.entries.items,
                .refreshing = refreshing,
            },
            json.omit_null_optionals,
        ) catch error.OutOfMemory;
    }

    /// Records why the last refresh produced nothing.
    pub fn setLastError(self: *Document, message: []const u8) Error!void {
        const owned = try self.allocator.dupe(u8, message);
        if (self.last_error) |previous| self.allocator.free(previous);
        self.last_error = owned;
    }

    /// Forgets the last refresh's failure, after one that worked.
    pub fn clearLastError(self: *Document) void {
        if (self.last_error) |previous| self.allocator.free(previous);
        self.last_error = null;
    }

    /// The observation for a name, if tracked.
    pub fn find(self: *Document, name: []const u8) ?*Observation {
        for (self.entries.items) |*entry| {
            if (std.mem.eql(u8, entry.name, name)) return entry;
        }
        return null;
    }

    /// Records the outcome of a probe and answers whether it is news.
    ///
    /// Nothing is marked as notified here: the notification has not happened
    /// yet.
    pub fn record(self: *Document, subject: Subject, probe: Probe, now: i64) Error!Outcome {
        // Any append invalidates pointers into the list, so the entry is
        // resolved once the list has stopped growing.
        const entry = try self.findOrCreate(subject);
        try self.setSource(entry, subject.source);
        if (subject.installed_version) |value| try self.replaceOwned(&entry.installed_version, value);
        if (subject.repo) |value| try self.replaceOwned(&entry.repo, value);
        if (subject.provider) |value| try self.replaceOwned(&entry.provider, value);
        if (subject.canal) |value| try self.replaceOwned(&entry.canal, value);
        if (subject.pkgname) |value| try self.replaceOwned(&entry.pkgname, value);

        switch (probe) {
            .failed => |reason| {
                try self.replaceOwned(&entry.last_error, reason);
                entry.consecutive_failures +|= 1;
                return .{ .seen = entry.last_seen, .answered = false, .is_news = false };
            },
            .seen => |offered| {
                if (entry.last_error) |value| {
                    self.allocator.free(value);
                    entry.last_error = null;
                }
                entry.consecutive_failures = 0;
                entry.last_checked = now;
                if (offered) |value| {
                    try self.replaceOwned(&entry.last_seen, value);
                }
                return .{
                    .seen = entry.last_seen,
                    .answered = true,
                    .is_news = isNews(entry.last_seen, entry.notified),
                };
            },
        }
    }

    /// Copies the observation for `name` out of `previous`, if it has one.
    ///
    /// This is what carries the notification decision across a refresh: the
    /// probes say what is on offer, `previous` says what the user has
    /// already been told. An observation whose subject has gone away is not
    /// copied, so a package removed from intent does not linger in the
    /// inventory from a stale note.
    pub fn inherit(self: *Document, previous: *const Document, name: []const u8) Error!void {
        const source = for (previous.entries.items) |entry| {
            if (std.mem.eql(u8, entry.name, name)) break entry;
        } else return;
        const entry = try self.findOrCreate(.{ .name = source.name, .source = source.source });
        if (source.installed_version) |value| try self.replaceOwned(&entry.installed_version, value);
        if (source.last_seen) |value| try self.replaceOwned(&entry.last_seen, value);
        if (source.notified) |value| try self.replaceOwned(&entry.notified, value);
        if (source.last_error) |value| try self.replaceOwned(&entry.last_error, value);
        if (source.repo) |value| try self.replaceOwned(&entry.repo, value);
        if (source.provider) |value| try self.replaceOwned(&entry.provider, value);
        if (source.canal) |value| try self.replaceOwned(&entry.canal, value);
        if (source.pkgname) |value| try self.replaceOwned(&entry.pkgname, value);
        entry.last_checked = source.last_checked;
        entry.consecutive_failures = source.consecutive_failures;
    }

    /// Marks a version as told.
    ///
    /// Called after the notification succeeded, never before.
    pub fn markNotified(self: *Document, name: []const u8, told: []const u8) Error!void {
        const entry = self.find(name) orelse return;
        try self.replaceOwned(&entry.notified, told);
    }

    fn findOrCreate(self: *Document, subject: Subject) Error!*Observation {
        if (self.find(subject.name)) |entry| return entry;

        const owned_name = try self.allocator.dupe(u8, subject.name);
        errdefer self.allocator.free(owned_name);
        const owned_source = try self.allocator.dupe(u8, subject.source);
        errdefer self.allocator.free(owned_source);

        try self.entries.append(self.allocator, .{
            .name = owned_name,
            .source = owned_source,
        });
        return &self.entries.items[self.entries.items.len - 1];
    }

    fn setSource(self: *Document, entry: *Observation, source: []const u8) Error!void {
        if (std.mem.eql(u8, entry.source, source)) return;
        const owned = try self.allocator.dupe(u8, source);
        self.allocator.free(entry.source);
        entry.source = owned;
    }

    fn replaceOwned(self: *Document, slot: *?[]const u8, value: []const u8) Error!void {
        const owned = try self.allocator.dupe(u8, value);
        if (slot.*) |previous| self.allocator.free(previous);
        slot.* = owned;
    }
};

/// The bytes of schema 2, in field order.
const Wire = struct {
    version: u32,
    checked_at: ?i64 = null,
    last_error: ?[]const u8 = null,
    entries: []const Observation,
    /// Carried by a response, never by the file.
    refreshing: ?bool = null,
};

fn byName(_: void, left: Observation, right: Observation) bool {
    return std.mem.order(u8, left.name, right.name) == .lt;
}

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

/// Reads a document of either schema.
fn parse(allocator: std.mem.Allocator, contents: []const u8) Error!Document {
    var parsed = std.json.parseFromSlice(std.json.Value, allocator, contents, .{}) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.Malformed,
    };
    defer parsed.deinit();

    const object = switch (parsed.value) {
        .object => |value| value,
        else => return error.Malformed,
    };

    var document = Document.init(allocator);
    errdefer document.deinit();

    const envelope = object.get("version") orelse {
        // Schema 1: a bare map of name to observation. A value that is not
        // an object is skipped rather than fatal -- the Python engine also
        // wrote entries it could not fill, and its readers skipped them.
        var iterator = object.iterator();
        while (iterator.next()) |entry| {
            const observation = switch (entry.value_ptr.*) {
                .object => |value| value,
                else => continue,
            };
            var parsed_entry = try observationFrom(allocator, entry.key_ptr.*, observation);
            errdefer parsed_entry.deinit(allocator);
            try document.entries.append(allocator, parsed_entry);
        }
        return document;
    };

    const written: u32 = switch (envelope) {
        .integer => |number| blk: {
            if (number < 0 or number > std.math.maxInt(u32)) return error.Malformed;
            break :blk @intCast(number);
        },
        else => return error.Malformed,
    };
    if (written > version) return error.UnsupportedVersion;
    if (written == 0) return error.Malformed;

    if (object.get("checked_at")) |value| {
        document.checked_at = switch (value) {
            .integer => |number| number,
            else => return error.Malformed,
        };
    }
    if (object.get("last_error")) |value| {
        document.last_error = switch (value) {
            .string => |text| try allocator.dupe(u8, text),
            else => return error.Malformed,
        };
    }

    const entries = switch (object.get("entries") orelse return error.Malformed) {
        .array => |value| value.items,
        else => return error.Malformed,
    };
    for (entries) |entry| {
        // Schema 2 is this module's own shape: an entry that is not an
        // object is a document that has been damaged, not one whose author
        // left a field empty.
        const observation = switch (entry) {
            .object => |value| value,
            else => return error.Malformed,
        };
        const name = switch (observation.get("name") orelse return error.Malformed) {
            .string => |text| text,
            else => return error.Malformed,
        };
        var parsed_entry = try observationFrom(allocator, name, observation);
        errdefer parsed_entry.deinit(allocator);
        try document.entries.append(allocator, parsed_entry);
    }
    return document;
}

/// One observation from a JSON object, owning every string it takes.
fn observationFrom(
    allocator: std.mem.Allocator,
    name: []const u8,
    object: std.json.ObjectMap,
) Error!Observation {
    const owned_name = try allocator.dupe(u8, name);
    const owned_source = blk: {
        errdefer allocator.free(owned_name);
        break :blk (try dupeText(allocator, object, &.{"source"})) orelse
            try allocator.dupe(u8, "");
    };

    var result: Observation = .{ .name = owned_name, .source = owned_source };
    errdefer result.deinit(allocator);

    // `latest_version` is what schema 1 called this: the newest version the
    // source offered. Reading it as `last_seen` is the migration.
    result.installed_version = try dupeText(allocator, object, &.{"installed_version"});
    result.last_seen = try dupeText(allocator, object, &.{ "last_seen", "latest_version" });
    result.notified = try dupeText(allocator, object, &.{"notified"});
    result.last_error = try dupeText(allocator, object, &.{"last_error"});
    result.repo = try dupeText(allocator, object, &.{"repo"});
    result.provider = try dupeText(allocator, object, &.{"provider"});
    result.canal = try dupeText(allocator, object, &.{"canal"});
    result.pkgname = try dupeText(allocator, object, &.{"pkgname"});

    // Schema 1 wrote `checked_at` as an ISO timestamp, which is a different
    // type and not a fact this document keeps: `last_checked` is Unix
    // seconds. The next probe fills it; until then the observation says the
    // source has not answered *under this document*, which is true.
    if (object.get("last_checked")) |value| {
        result.last_checked = switch (value) {
            .integer => |number| number,
            else => null,
        };
    }
    if (object.get("consecutive_failures")) |value| {
        result.consecutive_failures = switch (value) {
            .integer => |number| @intCast(std.math.clamp(
                number,
                0,
                @as(i64, std.math.maxInt(u32)),
            )),
            else => 0,
        };
    }
    return result;
}

/// The first of `keys` that carries a non-empty string, copied.
fn dupeText(
    allocator: std.mem.Allocator,
    object: std.json.ObjectMap,
    keys: []const []const u8,
) Error!?[]const u8 {
    for (keys) |key| {
        const value = object.get(key) orelse continue;
        switch (value) {
            .string => |text| if (text.len > 0) return try allocator.dupe(u8, text),
            else => {},
        }
    }
    return null;
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
/// The file need not exist: several tests are about absent documents and
/// directories that save has to create.
fn scratchPath(temporary: *testing.TmpDir, name: []const u8) ![]u8 {
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    return try std.fs.path.join(testing.allocator, &.{ buffer[0..length], name });
}

test "load: a missing document is empty, not an error" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var document = try Document.load(testing.allocator, testing.io, path);
    defer document.deinit();
    try testing.expectEqual(@as(usize, 0), document.entries.items.len);
}

test "save and load: a document round-trips" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var document = Document.init(testing.allocator);
    defer document.deinit();
    document.checked_at = 1_700_000_100;
    _ = try document.record(.{
        .name = "CADCraft",
        .source = "github",
        .installed_version = "0.8.0",
        .repo = "someone/CADCraft",
        .provider = "github",
        .canal = "stable",
        .pkgname = "cadcraft-bin",
    }, .{ .seen = "0.9.0" }, 1_700_000_000);
    try document.markNotified("CADCraft", "0.8.0");

    try document.save(testing.io, path);

    var reloaded = try Document.load(testing.allocator, testing.io, path);
    defer reloaded.deinit();

    try testing.expectEqual(@as(i64, 1_700_000_100), reloaded.checked_at.?);
    try testing.expectEqual(@as(usize, 1), reloaded.entries.items.len);
    const entry = reloaded.find("CADCraft").?;
    try testing.expectEqualStrings("github", entry.source);
    try testing.expectEqualStrings("0.8.0", entry.installed_version.?);
    try testing.expectEqualStrings("0.9.0", entry.last_seen.?);
    try testing.expectEqualStrings("0.8.0", entry.notified.?);
    try testing.expectEqualStrings("someone/CADCraft", entry.repo.?);
    try testing.expectEqual(@as(i64, 1_700_000_000), entry.last_checked.?);
    try testing.expectEqual(@as(u32, 0), entry.consecutive_failures);
    try testing.expect(entry.hasUpdate());
}

test "save: the envelope carries the schema and a null optional is omitted" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var document = Document.init(testing.allocator);
    defer document.deinit();
    _ = try document.record(.{ .name = "Fresh", .source = "github" }, .{ .seen = "1.0.0" }, 1_700_000_000);
    try document.save(testing.io, path);

    const contents = try readFile(testing.allocator, testing.io, path);
    defer testing.allocator.free(contents);
    try testing.expect(std.mem.indexOf(u8, contents, "\"version\":2") != null);
    try testing.expect(std.mem.indexOf(u8, contents, "\"notified\"") == null);
    try testing.expect(std.mem.indexOf(u8, contents, "null") == null);
    try testing.expect(std.mem.indexOf(u8, contents, "\"last_seen\":\"1.0.0\"") != null);
}

test "toJson: a response says a refresh started, the file never does" {
    var temporary = testing.tmpDir(.{}); 
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var document = Document.init(testing.allocator);
    defer document.deinit();
    _ = try document.record(.{ .name = "App", .source = "github" }, .{ .seen = "1.0.0" }, 100);

    const answer = try document.toJson(testing.allocator, true);
    defer testing.allocator.free(answer);
    try testing.expect(std.mem.indexOf(u8, answer, "\"refreshing\":true") != null);

    try document.save(testing.io, path);
    const stored = try readFile(testing.allocator, testing.io, path);
    defer testing.allocator.free(stored);
    try testing.expect(std.mem.indexOf(u8, stored, "refreshing") == null);
}

test "setLastError: a refresh that produced nothing is written down and cleared" {
    var document = Document.init(testing.allocator);
    defer document.deinit();

    try document.setLastError("the intent is malformed");
    try testing.expectEqualStrings("the intent is malformed", document.last_error.?);
    try document.setLastError("the base is locked");
    try testing.expectEqualStrings("the base is locked", document.last_error.?);

    document.clearLastError();
    try testing.expect(document.last_error == null);
}

test "record: a first sighting is news, and a repeat is not" {
    var document = Document.init(testing.allocator);
    defer document.deinit();
    const subject: Subject = .{ .name = "App", .source = "github" };

    const first = try document.record(subject, .{ .seen = "1.0.0" }, 100);
    try testing.expect(first.answered);
    try testing.expect(first.is_news);

    try document.markNotified("App", "1.0.0");

    const repeated = try document.record(subject, .{ .seen = "1.0.0" }, 200);
    try testing.expect(!repeated.is_news);

    const newer = try document.record(subject, .{ .seen = "1.1.0" }, 300);
    try testing.expect(newer.is_news);
}

test "hasUpdate: a forge tag carrying a v is an upgrade, not a wall" {
    var document = Document.init(testing.allocator);
    defer document.deinit();

    // The live probe of 2026-10-28 recorded `v0.9.0` against an installed
    // `0.8.1` and reported no update: the `v` made the tag un-orderable, and
    // the document answered "nothing new" about a real upgrade.
    _ = try document.record(
        .{ .name = "App", .source = "github", .installed_version = "0.8.1" },
        .{ .seen = "v0.9.0" },
        100,
    );
    try testing.expect(document.find("App").?.hasUpdate());
}

test "record: a forge rolling back to a prerelease is not an upgrade" {
    var document = Document.init(testing.allocator);
    defer document.deinit();
    const subject: Subject = .{ .name = "App", .source = "github" };

    _ = try document.record(subject, .{ .seen = "1.7.4" }, 100);
    try document.markNotified("App", "1.7.4");

    // The forge's "latest" flips to its own prerelease of the same release.
    // Telling a user on 1.7.4 to "update" to 1.7.4-rc1 is the bug the
    // release-tag ordering exists to prevent.
    const outcome = try document.record(subject, .{ .seen = "1.7.4-rc1" }, 200);
    try testing.expect(!outcome.is_news);

    const rolled_back = try document.record(subject, .{ .seen = "1.6.0" }, 300);
    try testing.expect(!rolled_back.is_news);
}

test "record: a version that cannot be ordered is still reported" {
    var document = Document.init(testing.allocator);
    defer document.deinit();
    const subject: Subject = .{ .name = "App", .source = "github" };

    _ = try document.record(subject, .{ .seen = "2024.10" }, 100);
    try document.markNotified("App", "2024.10");

    const outcome = try document.record(subject, .{ .seen = "nightly" }, 200);
    try testing.expect(outcome.is_news);
    // ...but not as an update: an unorderable pair cannot be compared.
    try testing.expect(!document.find("App").?.hasUpdate());
}

test "record: a failed probe keeps the last seen version and counts the failure" {
    var document = Document.init(testing.allocator);
    defer document.deinit();
    const subject: Subject = .{ .name = "App", .source = "github" };

    _ = try document.record(subject, .{ .seen = "2.0.0" }, 100);
    const failed = try document.record(subject, .{ .failed = "HTTP 503" }, 200);

    try testing.expect(!failed.answered);
    try testing.expect(!failed.is_news);
    try testing.expectEqualStrings("2.0.0", failed.seen.?);

    const entry = document.find("App").?;
    try testing.expectEqualStrings("HTTP 503", entry.last_error.?);
    try testing.expectEqual(@as(u32, 1), entry.consecutive_failures);
    // A failure is not a check: the timestamp still says when the source
    // last answered.
    try testing.expectEqual(@as(i64, 100), entry.last_checked.?);

    _ = try document.record(subject, .{ .failed = "HTTP 503" }, 300);
    try testing.expectEqual(@as(u32, 2), document.find("App").?.consecutive_failures);
}

test "record: the first success after failures clears the error" {
    var document = Document.init(testing.allocator);
    defer document.deinit();
    const subject: Subject = .{ .name = "App", .source = "github" };

    _ = try document.record(subject, .{ .failed = "timeout" }, 100);
    try testing.expectEqual(@as(u32, 1), document.find("App").?.consecutive_failures);

    _ = try document.record(subject, .{ .seen = "3.0.0" }, 200);
    const entry = document.find("App").?;
    try testing.expect(entry.last_error == null);
    try testing.expectEqual(@as(u32, 0), entry.consecutive_failures);
    try testing.expectEqual(@as(i64, 200), entry.last_checked.?);
}

test "record: a source that answers without a version is not news" {
    var document = Document.init(testing.allocator);
    defer document.deinit();

    const outcome = try document.record(.{ .name = "App", .source = "static_url" }, .{ .seen = null }, 100);
    try testing.expect(outcome.answered);
    try testing.expect(!outcome.is_news);
    try testing.expect(outcome.seen == null);
}

test "record: an empty probe with a stored version keeps the stored one" {
    var document = Document.init(testing.allocator);
    defer document.deinit();
    const subject: Subject = .{ .name = "App", .source = "github" };

    _ = try document.record(subject, .{ .seen = "1.0.0" }, 100);
    const outcome = try document.record(subject, .{ .seen = null }, 200);
    try testing.expectEqualStrings("1.0.0", outcome.seen.?);
    try testing.expect(outcome.is_news);
}

test "markNotified: an untracked name is a no-op, not a crash" {
    var document = Document.init(testing.allocator);
    defer document.deinit();
    try document.markNotified("Never seen", "1.0.0");
    try testing.expectEqual(@as(usize, 0), document.entries.items.len);
}

test "record: a source change is recorded, not ignored" {
    var document = Document.init(testing.allocator);
    defer document.deinit();

    _ = try document.record(.{ .name = "App", .source = "github" }, .{ .seen = "1.0.0" }, 100);
    _ = try document.record(.{ .name = "App", .source = "gitlab" }, .{ .seen = "1.0.0" }, 200);
    try testing.expectEqualStrings("gitlab", document.find("App").?.source);
    // The name is the key: a source change must not create a second entry.
    try testing.expectEqual(@as(usize, 1), document.entries.items.len);
}

test "record: the subject's own facts are recorded with the probe's answer" {
    var document = Document.init(testing.allocator);
    defer document.deinit();

    _ = try document.record(.{
        .name = "artcraft-launcher",
        .source = "github",
        .installed_version = "0.8.1",
        .repo = "storytold/craft-launcher",
        .provider = "github",
        .canal = "stable",
        .pkgname = "artcraft-launcher-bin",
    }, .{ .seen = "0.9.0" }, 100);

    const entry = document.find("artcraft-launcher").?;
    try testing.expectEqualStrings("0.8.1", entry.installed_version.?);
    try testing.expectEqualStrings("storytold/craft-launcher", entry.repo.?);
    try testing.expectEqualStrings("artcraft-launcher-bin", entry.pkgname.?);
    try testing.expect(entry.hasUpdate());
    try testing.expect(entry.hasNews());
}

test "inherit: the notification decision survives a refresh, the observation follows" {
    var previous = Document.init(testing.allocator);
    defer previous.deinit();
    _ = try previous.record(.{ .name = "App", .source = "github" }, .{ .seen = "1.0.0" }, 100);
    try previous.markNotified("App", "1.0.0");

    var next = Document.init(testing.allocator);
    defer next.deinit();
    try next.inherit(&previous, "App");
    try testing.expect(next.find("App") != null);

    // The probe offers 1.0.0 again: already told, so not news.
    const outcome = try next.record(.{ .name = "App", .source = "github" }, .{ .seen = "1.0.0" }, 200);
    try testing.expect(!outcome.is_news);
    try testing.expectEqual(@as(i64, 200), next.find("App").?.last_checked.?);
    // The decision survived the refresh; only the observation moved.
    try testing.expectEqualStrings("1.0.0", next.find("App").?.notified.?);
}

test "inherit: an observation with no subject any more is not carried" {
    var previous = Document.init(testing.allocator);
    defer previous.deinit();
    _ = try previous.record(.{ .name = "Gone", .source = "github" }, .{ .seen = "1.0.0" }, 100);

    var next = Document.init(testing.allocator);
    defer next.deinit();
    try next.inherit(&previous, "Never tracked");
    try testing.expectEqual(@as(usize, 0), next.entries.items.len);
}

test "load: schema 1, the bare map, is migrated instead of thrown away" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"artcraft-launcher": {"installed_version": "0.8.1", "repo": "storytold/craft-launcher", "provider": "github", "canal": "stable", "pkgname": "artcraft-launcher-bin", "latest_version": "0.9.0", "update_available": true, "checked_at": "2026-10-28T18:30:00Z"}}
        ,
    });

    var document = try Document.load(testing.allocator, testing.io, path);
    defer document.deinit();

    try testing.expectEqual(@as(usize, 1), document.entries.items.len);
    const entry = document.find("artcraft-launcher").?;
    try testing.expectEqualStrings("0.8.1", entry.installed_version.?);
    try testing.expectEqualStrings("0.9.0", entry.last_seen.?);
    try testing.expectEqualStrings("storytold/craft-launcher", entry.repo.?);
    try testing.expect(entry.hasUpdate());
    // The schema-1 timestamp is an ISO string, which this document does not
    // keep; the next probe fills `last_checked`.
    try testing.expect(entry.last_checked == null);
}

test "load: schema 1 with a non-object entry skips it, keeping the rest" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"broken": "not an observation", "kept": {"repo": "a/b"}}
        ,
    });

    var document = try Document.load(testing.allocator, testing.io, path);
    defer document.deinit();
    try testing.expectEqual(@as(usize, 1), document.entries.items.len);
    try testing.expect(document.find("kept") != null);
}

test "load: a document from a future schema is refused, not misread" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data = "{\"version\":99,\"entries\":[]}",
    });

    try testing.expectError(error.UnsupportedVersion, Document.load(testing.allocator, testing.io, path));
}

test "load: the previous daemon's registry envelope is read as schema 1" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"version":1,"entries":[{"name":"App","source":"github","last_seen":"1.0.0","notified":"0.9.0"}]}
        ,
    });

    var document = try Document.load(testing.allocator, testing.io, path);
    defer document.deinit();
    const entry = document.find("App").?;
    try testing.expectEqualStrings("1.0.0", entry.last_seen.?);
    try testing.expectEqualStrings("0.9.0", entry.notified.?);
}

test "load: a malformed document is refused rather than silently emptied" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data = "not json",
    });

    try testing.expectError(error.Malformed, Document.load(testing.allocator, testing.io, path));
}

test "load: an unknown field does not cost the user their document" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"version":2,"entries":[{"name":"App","source":"github","last_seen":"1.0.0","future_field":true}]}
        ,
    });

    var document = try Document.load(testing.allocator, testing.io, path);
    defer document.deinit();
    try testing.expectEqualStrings("1.0.0", document.find("App").?.last_seen.?);
}

test "load: schema 2 with a damaged entry is refused, not half-read" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data = "{\"version\":2,\"entries\":[{\"source\":\"github\"}]}",
    });

    try testing.expectError(error.Malformed, Document.load(testing.allocator, testing.io, path));
}

test "save: the parent directory is created" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "state/releases.json");
    defer testing.allocator.free(path);

    var document = Document.init(testing.allocator);
    defer document.deinit();
    _ = try document.record(.{ .name = "App", .source = "github" }, .{ .seen = "1.0.0" }, 100);

    try document.save(testing.io, path);

    var reloaded = try Document.load(testing.allocator, testing.io, path);
    defer reloaded.deinit();
    try testing.expectEqual(@as(usize, 1), reloaded.entries.items.len);
}

test "save: no staging file is left behind" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var document = Document.init(testing.allocator);
    defer document.deinit();
    try document.save(testing.io, path);

    var directory = try temporary.dir.openDir(testing.io, ".", .{ .iterate = true });
    defer directory.close(testing.io);
    var names = directory.iterate();
    while (try names.next(testing.io)) |entry| {
        try testing.expect(std.mem.indexOf(u8, entry.name, ".tmp-") == null);
    }
}

test "save: entries are written in name order, whatever order they arrived in" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratchPath(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var document = Document.init(testing.allocator);
    defer document.deinit();
    _ = try document.record(.{ .name = "Zebra", .source = "github" }, .{ .seen = "1.0.0" }, 100);
    _ = try document.record(.{ .name = "Ant", .source = "github" }, .{ .seen = "1.0.0" }, 100);
    try document.save(testing.io, path);

    const contents = try readFile(testing.allocator, testing.io, path);
    defer testing.allocator.free(contents);
    const ant = std.mem.indexOf(u8, contents, "Ant").?;
    const zebra = std.mem.indexOf(u8, contents, "Zebra").?;
    try testing.expect(ant < zebra);
}
