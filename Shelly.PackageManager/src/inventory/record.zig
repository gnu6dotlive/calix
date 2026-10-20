//! The normalised package record the inventory serves.
//!
//! One flat record for every ecosystem, with optional fields rather than a
//! tagged union. A union would make every new ecosystem a breaking change
//! for the consumers: the schema would have to grow a variant, and a
//! consumer that did not know the new variant would fail to parse the whole
//! document. A flat record lets the daemon add a field nobody reads, and
//! lets the frontend ignore what it does not know.
//!
//! The `Source` tags are the exact strings the frontend already uses for
//! `source_type`, so the two sides agree without a translation table.

const std = @import("std");
const shared_json = @import("../shared/json.zig");

/// The ecosystem a record came from.
pub const Source = enum {
    ALPM,
    AUR,
    Flatpak,
    AppImage,
    Cargo,
    NPM,
    Release,
};

/// A normalised package.
pub const Record = struct {
    /// The package name, as the source reports it.
    name: []const u8,
    /// The installed version.
    version: []const u8,
    /// A human-readable description. Empty when the source has none.
    description: []const u8 = "",
    /// The ecosystem this record came from.
    source_type: Source,
    /// The repository, registry or remote the package comes from.
    repository_or_remote: []const u8 = "",
    /// Whether the package is present on the system.
    is_installed: bool = true,
    /// Whether a newer version is available.
    has_update: bool = false,
    /// The available version, when `has_update` is set.
    new_version: ?[]const u8 = null,

    // ---- Optional detail, per ecosystem ------------------------------------

    /// The installed size in bytes, when the source reports one.
    size_on_disk: ?u64 = null,
    /// The icon name, when the source exposes one.
    icon_name: ?[]const u8 = null,
    /// The path of the artefact on disk, when there is one.
    path: ?[]const u8 = null,
    /// The URL the update source is polled at.
    update_url: ?[]const u8 = null,
    /// The forge owner, for release-managed packages.
    repo_owner: ?[]const u8 = null,
    /// The forge repository, for release-managed packages.
    repo_name: ?[]const u8 = null,
    /// Whether prereleases are accepted for the update source.
    allow_prerelease: bool = false,
    /// Whether an NPM package's version is overridden locally.
    overridden: bool = false,
    /// The binaries a Cargo crate installs.
    binaries: []const []const u8 = &.{},

    /// Serialises the record.
    pub fn toJson(self: Record, allocator: std.mem.Allocator) ![]u8 {
        return std.json.Stringify.valueAlloc(allocator, self, json_options);
    }

    pub fn deinit(self: *Record, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.version);
        allocator.free(self.description);
        allocator.free(self.repository_or_remote);
        if (self.new_version) |value| allocator.free(value);
        if (self.icon_name) |value| allocator.free(value);
        if (self.path) |value| allocator.free(value);
        if (self.update_url) |value| allocator.free(value);
        if (self.repo_owner) |value| allocator.free(value);
        if (self.repo_name) |value| allocator.free(value);
        for (self.binaries) |binary| allocator.free(binary);
        allocator.free(self.binaries);
        self.* = undefined;
    }
};

/// Releases every string a record owns.
pub fn deinitSlice(allocator: std.mem.Allocator, records: []Record) void {
    for (records) |*record| record.deinit(allocator);
    allocator.free(records);
}

/// The version of the snapshot format.
///
/// The frontend refuses a snapshot whose version it does not know, rather
/// than rendering a document it only half understands.
pub const snapshot_version = 1;

/// The options the inventory is written with.
///
/// A null optional is omitted rather than written as `null`: the document
/// stays compact, and a consumer that does not know a field simply does
/// not see it.
pub const json_options: std.json.Stringify.Options = shared_json.omit_null_optionals;

/// The document the daemon serves.
pub const Snapshot = struct {
    /// The format version.
    version: u32 = snapshot_version,
    /// When the snapshot was produced, in Unix seconds.
    generated_at: i64,
    /// The normalised packages.
    packages: []const Record,

    /// Serialises the snapshot.
    pub fn toJson(self: Snapshot, allocator: std.mem.Allocator) ![]u8 {
        return std.json.Stringify.valueAlloc(allocator, self, json_options);
    }
};

test "Source: the tags are the strings the frontend uses" {
    try std.testing.expectEqualStrings("ALPM", @tagName(Source.ALPM));
    try std.testing.expectEqualStrings("AUR", @tagName(Source.AUR));
    try std.testing.expectEqualStrings("Flatpak", @tagName(Source.Flatpak));
    try std.testing.expectEqualStrings("AppImage", @tagName(Source.AppImage));
    try std.testing.expectEqualStrings("Cargo", @tagName(Source.Cargo));
    try std.testing.expectEqualStrings("NPM", @tagName(Source.NPM));
    try std.testing.expectEqualStrings("Release", @tagName(Source.Release));
}

test "Record: serialises to the flat shape with null optionals omitted" {
    var record = Record{
        .name = try std.testing.allocator.dupe(u8, "ripgrep"),
        .version = try std.testing.allocator.dupe(u8, "14.1.1"),
        .source_type = .Cargo,
        .repository_or_remote = try std.testing.allocator.dupe(u8, "crates.io"),
    };
    defer record.deinit(std.testing.allocator);

    const json = try record.toJson(std.testing.allocator);
    defer std.testing.allocator.free(json);

    try std.testing.expect(std.mem.indexOf(u8, json, "\"source_type\":\"Cargo\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, json, "\"name\":\"ripgrep\"") != null);
    // Optional fields that are unset are omitted, not null.
    try std.testing.expect(std.mem.indexOf(u8, json, "icon_name") == null);
    try std.testing.expect(std.mem.indexOf(u8, json, "new_version") == null);
    // Required fields are always present.
    try std.testing.expect(std.mem.indexOf(u8, json, "\"is_installed\"") != null);
    try std.testing.expect(std.mem.indexOf(u8, json, "\"has_update\"") != null);
}

// The exact shape the frontend depends on.
//
// The field names here are the frontend's vocabulary: `source_type` and
// `repository_or_remote` are the names `UnifiedPackage` already uses, so
// the frontend can consume a snapshot without a translation layer. A
// change to this shape is a change to the contract and has to be made in
// both places at once.
test "Record: the serialised shape is the contract the frontend reads" {
    var record = Record{
        .name = try std.testing.allocator.dupe(u8, "cadcraft"),
        .version = try std.testing.allocator.dupe(u8, "0.8.0"),
        .description = try std.testing.allocator.dupe(u8, "d"),
        .source_type = .AppImage,
        .repository_or_remote = try std.testing.allocator.dupe(u8, "cadcraft"),
        .size_on_disk = 42,
    };
    defer record.deinit(std.testing.allocator);

    const json = try record.toJson(std.testing.allocator);
    defer std.testing.allocator.free(json);

    const expected =
        \\{"name":"cadcraft","version":"0.8.0","description":"d","source_type":"AppImage","repository_or_remote":"cadcraft","is_installed":true,"has_update":false,"size_on_disk":42,"allow_prerelease":false,"overridden":false,"binaries":[]}
    ;
    try std.testing.expectEqualStrings(expected, json);
}
