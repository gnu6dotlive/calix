//! The release intent: which packages the user asked to follow.
//!
//! `~/.config/calix/releases.json`, written by the user through
//! `calix registry add` and read by the daemon. The daemon never writes it:
//! intent is a decision, and a service that rewrites the user's decision is a
//! service that overrules it. What the daemon learns about those packages
//! goes into its own document (`document.zig`).
//!
//! Schema 1 is the map the frozen Python engine writes: name to an object of
//! hint fields. A missing or unreadable file is an empty intent -- a machine
//! that tracks nothing is the state of a fresh installation, not a fault --
//! but a *malformed* file is an error the caller can name, because silently
//! treating it as empty would drop every package the user asked to follow.

const std = @import("std");

pub const Error = error{
    /// The intent exists but could not be read.
    Unreadable,
    /// The intent is not the shape this module reads.
    Malformed,
    OutOfMemory,
};

/// One package the user asked to follow.
pub const Entry = struct {
    /// The key in the file, and the name every consumer knows the package
    /// by.
    name: []const u8,
    /// `owner/name`, as the forge spells it.
    repo: ?[]const u8 = null,
    /// `github`, `gitlab`, `gitea`, `codeberg`, `forgejo`.
    provider: ?[]const u8 = null,
    /// The forge's host, for the providers that are not one site.
    host: ?[]const u8 = null,
    /// `stable`, `beta`, `canary`, `nightly`.
    canal: ?[]const u8 = null,
    /// The package the build installs, when it installs one.
    pkgname: ?[]const u8 = null,
    /// What the user's own record says is installed. The daemon prefers the
    /// package database's answer, and falls back to this.
    installed_version: ?[]const u8 = null,

    pub fn deinit(self: *Entry, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        for ([_]*?[]const u8{
            &self.repo,
            &self.provider,
            &self.host,
            &self.canal,
            &self.pkgname,
            &self.installed_version,
        }) |slot| {
            if (slot.*) |value| allocator.free(value);
        }
        self.* = undefined;
    }
};

/// Reads the intent. A missing file is an empty list, not an error.
pub fn load(
    allocator: std.mem.Allocator,
    io: std.Io,
    path: []const u8,
) Error!std.ArrayList(Entry) {
    var list: std.ArrayList(Entry) = .empty;
    errdefer freeAll(allocator, &list);

    const contents = readFile(allocator, io, path) catch |err| switch (err) {
        error.FileNotFound => return list,
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.Unreadable,
    };
    defer allocator.free(contents);

    var parsed = std.json.parseFromSlice(std.json.Value, allocator, contents, .{}) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.Malformed,
    };
    defer parsed.deinit();

    const object = switch (parsed.value) {
        .object => |value| value,
        else => return error.Malformed,
    };

    var iterator = object.iterator();
    while (iterator.next()) |item| {
        // An entry that is not an object carries nothing to follow. It is
        // skipped rather than fatal, the same rule the document applies to
        // schema 1: one empty line must not hide every other package.
        const fields = switch (item.value_ptr.*) {
            .object => |value| value,
            else => continue,
        };
        var entry: Entry = .{ .name = try allocator.dupe(u8, item.key_ptr.*) };
        errdefer entry.deinit(allocator);
        entry.repo = try dupeText(allocator, fields, "repo");
        entry.provider = try dupeText(allocator, fields, "provider");
        entry.host = try dupeText(allocator, fields, "host");
        entry.canal = try dupeText(allocator, fields, "canal");
        entry.pkgname = try dupeText(allocator, fields, "pkgname");
        entry.installed_version = try dupeText(allocator, fields, "installed_version");

        try list.append(allocator, entry);
    }
    return list;
}

/// Releases every entry the list holds.
pub fn freeAll(allocator: std.mem.Allocator, list: *std.ArrayList(Entry)) void {
    for (list.items) |*entry| entry.deinit(allocator);
    list.deinit(allocator);
}

fn dupeText(
    allocator: std.mem.Allocator,
    object: std.json.ObjectMap,
    key: []const u8,
) Error!?[]const u8 {
    const value = object.get(key) orelse return null;
    return switch (value) {
        .string => |text| if (text.len > 0) try allocator.dupe(u8, text) else null,
        else => null,
    };
}

fn readFile(allocator: std.mem.Allocator, io: std.Io, path: []const u8) ![]u8 {
    var file = try std.Io.Dir.cwd().openFile(io, path, .{});
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var reader = file.reader(io, &buffer);
    return try reader.interface.allocRemaining(allocator, .unlimited);
}

const testing = std.testing;

fn scratch(temporary: *testing.TmpDir, name: []const u8) ![]u8 {
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const length = try temporary.dir.realPath(testing.io, &buffer);
    return try std.fs.path.join(testing.allocator, &.{ buffer[0..length], name });
}

test "load: a missing intent is an empty list, not an error" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratch(&temporary, "releases.json");
    defer testing.allocator.free(path);

    var list = try load(testing.allocator, testing.io, path);
    defer freeAll(testing.allocator, &list);
    try testing.expectEqual(@as(usize, 0), list.items.len);
}

test "load: the schema the Python engine writes is read field for field" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratch(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"artcraft-launcher": {"repo": "storytold/craft-launcher", "provider": "github", "host": "github.com", "canal": "stable", "pkgname": "artcraft-launcher-bin", "pkgbuild_path": "/tmp/PKGBUILD", "status": "managed", "installed_version": "0.8.1"}}
        ,
    });

    var list = try load(testing.allocator, testing.io, path);
    defer freeAll(testing.allocator, &list);

    try testing.expectEqual(@as(usize, 1), list.items.len);
    const entry = list.items[0];
    try testing.expectEqualStrings("artcraft-launcher", entry.name);
    try testing.expectEqualStrings("storytold/craft-launcher", entry.repo.?);
    try testing.expectEqualStrings("github", entry.provider.?);
    try testing.expectEqualStrings("github.com", entry.host.?);
    try testing.expectEqualStrings("stable", entry.canal.?);
    try testing.expectEqualStrings("artcraft-launcher-bin", entry.pkgname.?);
    try testing.expectEqualStrings("0.8.1", entry.installed_version.?);
}

test "load: a malformed intent is named, not read as empty" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratch(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{ .sub_path = "releases.json", .data = "not json" });
    try testing.expectError(error.Malformed, load(testing.allocator, testing.io, path));
}

test "load: an entry that is not an object is skipped, the rest kept" {
    var temporary = testing.tmpDir(.{});
    defer temporary.cleanup();
    const path = try scratch(&temporary, "releases.json");
    defer testing.allocator.free(path);

    try temporary.dir.writeFile(testing.io, .{
        .sub_path = "releases.json",
        .data =
        \\{"broken": "not an entry", "kept": {"repo": "a/b"}}
        ,
    });

    var list = try load(testing.allocator, testing.io, path);
    defer freeAll(testing.allocator, &list);
    try testing.expectEqual(@as(usize, 1), list.items.len);
    try testing.expectEqualStrings("kept", list.items[0].name);
}
