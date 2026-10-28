//! Release-tracked packages, as the inventory serves them.
//!
//! # Transitional
//!
//! The set is currently published by the frozen Python engine as
//! `<state>/releases.json`. This module is the only reader left: the
//! frontend no longer reads the file, it reads the daemon. When the Zig
//! release watcher lands it produces these records itself, and this file is
//! deleted — along with the Python state it reads.
//!
//! The file is an observation, not intent: `~/.config/calix/releases.json`
//! says which packages are tracked, and this one says what was last seen for
//! each. The difference matters because a package that was removed leaves an
//! observation behind, and the inventory must not resurrect it from a stale
//! note. It is read as-is for that reason: the watcher is what owns intent.
//!
//! A missing, unreadable or malformed document contributes nothing. The
//! inventory is a view of what is there, not a checklist of what should be,
//! and a frontend that refuses to render because one ecosystem is unreadable
//! is worse than one that renders the rest.

const std = @import("std");
const record = @import("../inventory/record.zig");

/// The name of the document in the state directory.
pub const file_name = "releases.json";

/// The largest document read. A release state file is a few kilobytes; a
/// bound keeps a corrupted file from being read into memory whole.
const max_document_bytes = 8 * 1024 * 1024;

/// Appends one record per tracked release in the state directory.
pub fn append(
    allocator: std.mem.Allocator,
    io: std.Io,
    state_directory: []const u8,
    records: *std.ArrayList(record.Record),
) error{OutOfMemory}!void {
    const parsed = (try loadDocument(allocator, io, state_directory)) orelse return;
    defer parsed.deinit();

    // The document is a map of tracked name to observation.
    const document = switch (parsed.value) {
        .object => |object| object,
        else => return,
    };

    var iterator = document.iterator();
    while (iterator.next()) |entry| {
        const observation = switch (entry.value_ptr.*) {
            .object => |object| object,
            else => continue,
        };
        try appendOne(allocator, records, entry.key_ptr.*, observation);
    }
}

/// How many packages the state directory says are tracked.
///
/// The daemon reports this in `status`, and the answer has to be the same one
/// `append` would produce: a status that counts a different set than the
/// inventory serves is a number a frontend cannot use. Counting here rather
/// than materialising records keeps that agreement structural instead of
/// making it two parsers that happen to match today.
///
/// An absent, unreadable or malformed document is zero — the same rule
/// `append` applies, so the two never disagree about a broken file.
pub fn count(
    allocator: std.mem.Allocator,
    io: std.Io,
    state_directory: []const u8,
) error{OutOfMemory}!usize {
    const parsed = (try loadDocument(allocator, io, state_directory)) orelse return 0;
    defer parsed.deinit();

    const document = switch (parsed.value) {
        .object => |object| object,
        else => return 0,
    };

    var total: usize = 0;
    var iterator = document.iterator();
    while (iterator.next()) |entry| {
        // An entry that is not an observation is not a tracked release:
        // `append` skips it, so it is not counted either.
        switch (entry.value_ptr.*) {
            .object => total += 1,
            else => {},
        }
    }
    return total;
}

/// The parsed document, or null when it is absent, unreadable or malformed.
///
/// Both readers above go through this so that they read the same bytes: two
/// independent parsers of one file is how a count and a list start disagreeing.
fn loadDocument(
    allocator: std.mem.Allocator,
    io: std.Io,
    state_directory: []const u8,
) error{OutOfMemory}!?std.json.Parsed(std.json.Value) {
    const path = try std.fs.path.join(allocator, &.{ state_directory, file_name });
    defer allocator.free(path);

    const text = std.Io.Dir.cwd().readFileAlloc(
        io,
        path,
        allocator,
        .limited(max_document_bytes),
    ) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        // Absent: nothing has been tracked yet. Unreadable: the document is
        // not ours to repair. Both contribute no records.
        else => return null,
    };
    defer allocator.free(text);

    return std.json.parseFromSlice(std.json.Value, allocator, text, .{}) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return null,
    };
}

fn appendOne(
    allocator: std.mem.Allocator,
    records: *std.ArrayList(record.Record),
    name: []const u8,
    observation: std.json.ObjectMap,
) error{OutOfMemory}!void {
    const provider = stringField(observation, "provider");
    const canal = stringField(observation, "canal");
    const installed = stringField(observation, "installed_version");
    const latest = stringField(observation, "latest_version");
    const update_available = boolField(observation, "update_available") orelse false;

    // A tracked package with no known version still belongs in the list; it
    // is a package the user asked to follow, not a row to hide.
    const version = installed orelse latest orelse "unknown";

    const description = if (provider != null and canal != null)
        try std.fmt.allocPrint(
            allocator,
            "Release-managed package — {s} channel, {s}",
            .{ provider.?, canal.? },
        )
    else
        try allocator.dupe(u8, "Release-managed package");
    errdefer allocator.free(description);

    const owned_name = try allocator.dupe(u8, name);
    errdefer allocator.free(owned_name);
    const owned_version = try allocator.dupe(u8, version);
    errdefer allocator.free(owned_version);
    const owned_repository = try allocator.dupe(
        u8,
        stringField(observation, "repo") orelse "",
    );
    errdefer allocator.free(owned_repository);
    // The field is documented as "the available version, when `has_update`
    // is set". Carrying a `new_version` equal to the installed one invites a
    // consumer to render an update that is not there.
    const owned_latest: ?[]const u8 = if (update_available) blk: {
        const value = latest orelse break :blk null;
        break :blk try allocator.dupe(u8, value);
    } else null;
    errdefer if (owned_latest) |value| allocator.free(value);
    const owned_provider: ?[]const u8 = if (provider) |value|
        try allocator.dupe(u8, value)
    else
        null;
    errdefer if (owned_provider) |value| allocator.free(value);
    const owned_canal: ?[]const u8 = if (canal) |value|
        try allocator.dupe(u8, value)
    else
        null;
    errdefer if (owned_canal) |value| allocator.free(value);
    const owned_pkgname: ?[]const u8 = if (stringField(observation, "pkgname")) |value|
        try allocator.dupe(u8, value)
    else
        null;
    errdefer if (owned_pkgname) |value| allocator.free(value);

    try records.append(allocator, .{
        .name = owned_name,
        .version = owned_version,
        .description = description,
        .source_type = .Release,
        .repository_or_remote = owned_repository,
        .has_update = update_available,
        .new_version = owned_latest,
        .provider = owned_provider,
        .canal = owned_canal,
        .pkgname = owned_pkgname,
    });
}

fn stringField(object: std.json.ObjectMap, key: []const u8) ?[]const u8 {
    const value = object.get(key) orelse return null;
    return switch (value) {
        .string => |text| if (text.len == 0) null else text,
        else => null,
    };
}

fn boolField(object: std.json.ObjectMap, key: []const u8) ?bool {
    const value = object.get(key) orelse return null;
    return switch (value) {
        .bool => |flag| flag,
        else => null,
    };
}

const testing = std.testing;

/// The document the Python engine writes, reduced to the keys this reads.
const sample =
    \\{
    \\  "artcraft-launcher": {
    \\    "installed_version": "0.5.1",
    \\    "repo": "storytold/craft-launcher",
    \\    "provider": "github",
    \\    "canal": "stable",
    \\    "pkgname": "artcraft-launcher-bin",
    \\    "latest_version": "0.5.2",
    \\    "update_available": true
    \\  }
    \\}
;

test "Tracked: a release observation becomes a record" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{ .sub_path = file_name, .data = sample });

    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    var records: std.ArrayList(record.Record) = .empty;
    defer {
        for (records.items) |*item| item.deinit(testing.allocator);
        records.deinit(testing.allocator);
    }
    try append(testing.allocator, testing.io, buffer[0..length], &records);

    try testing.expectEqual(@as(usize, 1), records.items.len);
    const first = records.items[0];
    try testing.expectEqualStrings("artcraft-launcher", first.name);
    try testing.expectEqualStrings("0.5.1", first.version);
    try testing.expectEqual(record.Source.Release, first.source_type);
    try testing.expectEqualStrings("storytold/craft-launcher", first.repository_or_remote);
    try testing.expectEqualStrings("github", first.provider.?);
    try testing.expectEqualStrings("stable", first.canal.?);
    try testing.expectEqualStrings("artcraft-launcher-bin", first.pkgname.?);
    try testing.expect(first.has_update);
    try testing.expectEqualStrings("0.5.2", first.new_version.?);
    try testing.expectEqualStrings(
        "Release-managed package — github channel, stable",
        first.description,
    );
}

test "Tracked: an absent document contributes nothing" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    var records: std.ArrayList(record.Record) = .empty;
    defer records.deinit(testing.allocator);
    try append(testing.allocator, testing.io, buffer[0..length], &records);

    try testing.expectEqual(@as(usize, 0), records.items.len);
}

test "Tracked: a malformed document contributes nothing" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{ .sub_path = file_name, .data = "{not json" });

    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    var records: std.ArrayList(record.Record) = .empty;
    defer records.deinit(testing.allocator);
    try append(testing.allocator, testing.io, buffer[0..length], &records);

    try testing.expectEqual(@as(usize, 0), records.items.len);
}

test "Tracked: a non-object entry is skipped, not rendered" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = file_name,
        .data = \\{"broken": "not an observation", "unknown-version": {"repo": "a/b"}}
        ,
    });

    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    var records: std.ArrayList(record.Record) = .empty;
    defer {
        for (records.items) |*item| item.deinit(testing.allocator);
        records.deinit(testing.allocator);
    }
    try append(testing.allocator, testing.io, buffer[0..length], &records);

    try testing.expectEqual(@as(usize, 1), records.items.len);
    // No installed or latest version: the row still exists, labelled so.
    try testing.expectEqualStrings("unknown", records.items[0].version);
    try testing.expect(records.items[0].new_version == null);
}

test "Tracked: the count agrees with the records, entry for entry" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{ .sub_path = file_name, .data = sample });

    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    var records: std.ArrayList(record.Record) = .empty;
    defer {
        for (records.items) |*item| item.deinit(testing.allocator);
        records.deinit(testing.allocator);
    }
    try append(testing.allocator, testing.io, buffer[0..length], &records);
    const counted = try count(testing.allocator, testing.io, buffer[0..length]);

    // The status a daemon reports and the list an inventory serves are the
    // same set, so these two numbers cannot drift apart.
    try testing.expectEqual(records.items.len, counted);
    try testing.expectEqual(@as(usize, 1), counted);
}

test "Tracked: count is zero when the document is absent" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    try testing.expectEqual(
        @as(usize, 0),
        try count(testing.allocator, testing.io, buffer[0..length]),
    );
}

test "Tracked: count is zero when the document is malformed" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{ .sub_path = file_name, .data = "{not json" });

    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    try testing.expectEqual(
        @as(usize, 0),
        try count(testing.allocator, testing.io, buffer[0..length]),
    );
}

test "Tracked: a non-object entry is not counted, because it is not a release" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = file_name,
        .data = \\{"broken": "not an observation", "unknown-version": {"repo": "a/b"}}
        ,
    });

    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);

    // `append` skips `broken`; a count that included it would report a
    // tracked package the inventory never shows.
    try testing.expectEqual(
        @as(usize, 1),
        try count(testing.allocator, testing.io, buffer[0..length]),
    );
}
