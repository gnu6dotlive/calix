//! Release-tracked packages, as the inventory serves them.
//!
//! The document belongs to the daemon (`document.zig`), which is its only
//! writer. This module is the inventory's view of it: one record per
//! observation, so a package the user follows appears in the same list as
//! everything else installed.
//!
//! It reads through `Document.load` rather than parsing the file itself. A
//! second parser of one document is how a count and a list start disagreeing
//! about what the document says, and the migration from schema 1 would then
//! have to be written twice — with the second copy being the one that rots.
//!
//! A document that is absent, unreadable or damaged contributes nothing. The
//! inventory is a view of what is there, not a checklist of what should be,
//! and a frontend that refuses to render because one ecosystem is unreadable
//! is worse than one that renders the rest.

const std = @import("std");
const record = @import("../inventory/record.zig");
const document = @import("document.zig");

/// Appends one record per tracked release in the state directory.
pub fn append(
    allocator: std.mem.Allocator,
    io: std.Io,
    state_directory: []const u8,
    records: *std.ArrayList(record.Record),
) error{OutOfMemory}!void {
    const path = try std.fs.path.join(allocator, &.{ state_directory, document.file_name });
    defer allocator.free(path);

    var loaded = document.Document.load(allocator, io, path) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        // Absent: nothing has been tracked yet. Damaged, or written by a
        // newer Calix: the inventory does not repair it and does not guess
        // at it. Both contribute no records.
        else => return,
    };
    defer loaded.deinit();

    for (loaded.entries.items) |observation| {
        try appendOne(allocator, records, observation);
    }
}

fn appendOne(
    allocator: std.mem.Allocator,
    records: *std.ArrayList(record.Record),
    observation: document.Observation,
) error{OutOfMemory}!void {
    // A tracked package with no known version still belongs in the list; it
    // is a package the user asked to follow, not a row to hide.
    const version = observation.installed_version orelse observation.last_seen orelse "unknown";

    const description = if (observation.provider != null and observation.canal != null)
        try std.fmt.allocPrint(
            allocator,
            "Release-managed package — {s} channel, {s}",
            .{ observation.provider.?, observation.canal.? },
        )
    else
        try allocator.dupe(u8, "Release-managed package");
    errdefer allocator.free(description);

    const owned_name = try allocator.dupe(u8, observation.name);
    errdefer allocator.free(owned_name);
    const owned_version = try allocator.dupe(u8, version);
    errdefer allocator.free(owned_version);
    const owned_repository = try allocator.dupe(u8, observation.repo orelse "");
    errdefer allocator.free(owned_repository);

    // Derived, never stored: the document keeps what the source offered and
    // what the base says is installed, and the comparison between them is a
    // rule that must have exactly one definition.
    const has_update = observation.hasUpdate();
    // `new_version` is documented as "the available version, when
    // `has_update` is set". Carrying one equal to the installed version
    // invites a consumer to render an update that is not there.
    const owned_latest: ?[]const u8 = if (has_update)
        try allocator.dupe(u8, observation.last_seen.?)
    else
        null;
    errdefer if (owned_latest) |value| allocator.free(value);

    const owned_provider: ?[]const u8 = if (observation.provider) |value|
        try allocator.dupe(u8, value)
    else
        null;
    errdefer if (owned_provider) |value| allocator.free(value);
    const owned_canal: ?[]const u8 = if (observation.canal) |value|
        try allocator.dupe(u8, value)
    else
        null;
    errdefer if (owned_canal) |value| allocator.free(value);
    const owned_pkgname: ?[]const u8 = if (observation.pkgname) |value|
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
        .has_update = has_update,
        .new_version = owned_latest,
        .provider = owned_provider,
        .canal = owned_canal,
        .pkgname = owned_pkgname,
    });
}

const testing = std.testing;

fn scratch(temporary: *testing.TmpDir) ![]u8 {
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    return testing.allocator.dupe(u8, buffer[0..length]);
}

fn collect(records: *std.ArrayList(record.Record)) void {
    for (records.items) |*item| item.deinit(testing.allocator);
    records.deinit(testing.allocator);
}

test "Tracked: one observation becomes one record" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = document.file_name,
        .data =
        \\{"version":2,"entries":[{"name":"artcraft-launcher","source":"github","installed_version":"0.5.1","last_seen":"0.5.2","repo":"storytold/craft-launcher","provider":"github","canal":"stable","pkgname":"artcraft-launcher-bin"}]}
        ,
    });
    const state = try scratch(&temporary);
    defer testing.allocator.free(state);

    var records: std.ArrayList(record.Record) = .empty;
    defer collect(&records);
    try append(testing.allocator, testing.io, state, &records);

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

test "Tracked: a source that offers nothing new is not rendered as an update" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    // The forge's latest is older than what is installed: a downgrade is not
    // an update, and `new_version` must not invite one.
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = document.file_name,
        .data =
        \\{"version":2,"entries":[{"name":"App","source":"github","installed_version":"2.0.0","last_seen":"1.9.0"}]}
        ,
    });
    const state = try scratch(&temporary);
    defer testing.allocator.free(state);

    var records: std.ArrayList(record.Record) = .empty;
    defer collect(&records);
    try append(testing.allocator, testing.io, state, &records);

    try testing.expectEqual(@as(usize, 1), records.items.len);
    try testing.expect(!records.items[0].has_update);
    try testing.expect(records.items[0].new_version == null);
}

test "Tracked: a document written before schema 2 is still served" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = document.file_name,
        .data =
        \\{"artcraft-launcher": {"installed_version": "0.5.1", "latest_version": "0.5.2", "update_available": true, "repo": "storytold/craft-launcher"}}
        ,
    });
    const state = try scratch(&temporary);
    defer testing.allocator.free(state);

    var records: std.ArrayList(record.Record) = .empty;
    defer collect(&records);
    try append(testing.allocator, testing.io, state, &records);

    try testing.expectEqual(@as(usize, 1), records.items.len);
    try testing.expectEqualStrings("0.5.1", records.items[0].version);
    try testing.expect(records.items[0].has_update);
    try testing.expectEqualStrings("0.5.2", records.items[0].new_version.?);
}

test "Tracked: an absent document contributes nothing" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const state = try scratch(&temporary);
    defer testing.allocator.free(state);

    var records: std.ArrayList(record.Record) = .empty;
    defer records.deinit(testing.allocator);
    try append(testing.allocator, testing.io, state, &records);

    try testing.expectEqual(@as(usize, 0), records.items.len);
}

test "Tracked: a damaged document contributes nothing, not half a document" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    // One entry is damaged. Serving the other would be the inventory
    // inventing a set the daemon never committed.
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = document.file_name,
        .data = \\{"version":2,"entries":[{"name":"App","source":"github"},{"source":"github"}]}
        ,
    });
    const state = try scratch(&temporary);
    defer testing.allocator.free(state);

    var records: std.ArrayList(record.Record) = .empty;
    defer records.deinit(testing.allocator);
    try append(testing.allocator, testing.io, state, &records);

    try testing.expectEqual(@as(usize, 0), records.items.len);
}

test "Tracked: a document from a future schema contributes nothing" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    try temporary.dir.writeFile(testing.io, .{
        .sub_path = document.file_name,
        .data = \\{"version":99,"entries":[{"name":"App","source":"github"}]}
        ,
    });
    const state = try scratch(&temporary);
    defer testing.allocator.free(state);

    var records: std.ArrayList(record.Record) = .empty;
    defer records.deinit(testing.allocator);
    try append(testing.allocator, testing.io, state, &records);

    try testing.expectEqual(@as(usize, 0), records.items.len);
}
