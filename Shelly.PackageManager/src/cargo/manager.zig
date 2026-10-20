//! The Cargo ecosystem: crates installed by cargo-install.
//!
//! Cargo keeps its own index of what it installed in
//! `~/.cargo/.crates2.json`, one entry per installed crate. The index is
//! authoritative — it is what `cargo uninstall` consults — so reading it is
//! better than inferring the inventory from the binaries in `~/.cargo/bin`,
//! which cannot say which crate a binary came from.

const std = @import("std");
const record = @import("../inventory/record.zig");

pub const Error = error{
    /// The index could not be read.
    IndexUnreadable,
    /// The index is not the shape cargo writes.
    IndexMalformed,
    /// An allocation failed.
    OutOfMemory,
    /// Cargo has never installed anything, so it has no index.
    FileNotFound,
};

/// The default location of the cargo index, relative to the home directory.
pub const default_index_path = ".cargo/.crates2.json";

pub const Options = struct {
    /// The path of the cargo index.
    index_path: []const u8 = default_index_path,
};

/// A crate installed by cargo.
pub const Crate = struct {
    name: []const u8,
    version: []const u8,
    /// The registry the crate came from.
    source: []const u8,
    /// The binaries the crate installs.
    binaries: []const []const u8,

    pub fn deinit(self: *Crate, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.version);
        allocator.free(self.source);
        for (self.binaries) |binary| allocator.free(binary);
        allocator.free(self.binaries);
        self.* = undefined;
    }
};

pub fn deinitSlice(allocator: std.mem.Allocator, crates: []Crate) void {
    for (crates) |*crate| crate.deinit(allocator);
    allocator.free(crates);
}

pub const Manager = struct {
    allocator: std.mem.Allocator,
    io: std.Io,
    options: Options,

    pub fn init(allocator: std.mem.Allocator, io: std.Io, options: Options) Manager {
        return .{ .allocator = allocator, .io = io, .options = options };
    }

    /// Lists the crates cargo has installed.
    ///
    /// A missing index is not an error: it simply means cargo has never
    /// installed anything.
    pub fn list(self: Manager) Error![]Crate {
        const contents = readIndex(self.allocator, self.io, self.options.index_path) catch |err| switch (err) {
            error.FileNotFound => return &.{},
            else => return err,
        };
        defer self.allocator.free(contents);

        return parseIndex(self.allocator, contents);
    }

    /// Appends the crates as normalised inventory records.
    ///
    /// The records are appended into the caller's list, so ownership of
    /// every string is unambiguous: the caller frees them, and the manager
    /// never hands out a slice whose lifetime overlaps another owner's.
    pub fn inventory(self: Manager, records: *std.ArrayList(record.Record)) Error!void {
        const crates = try self.list();
        defer deinitSlice(self.allocator, crates);

        for (crates) |crate| {
            var binaries: std.ArrayList([]const u8) = .empty;
            errdefer {
                for (binaries.items) |binary| self.allocator.free(binary);
                binaries.deinit(self.allocator);
            }
            for (crate.binaries) |binary| {
                try binaries.append(self.allocator, try self.allocator.dupe(u8, binary));
            }

            const description = if (binaries.items.len == 0)
                "Cargo crate"
            else
                try std.fmt.allocPrint(
                    self.allocator,
                    "Cargo crate — binaries: {s}",
                    .{try joinBinaries(self.allocator, binaries.items)},
                );
            errdefer self.allocator.free(description);

            try records.append(self.allocator, .{
                .name = try self.allocator.dupe(u8, crate.name),
                .version = try self.allocator.dupe(u8, crate.version),
                .description = description,
                .source_type = .Cargo,
                .repository_or_remote = try self.allocator.dupe(u8, crate.source),
                .binaries = try binaries.toOwnedSlice(self.allocator),
            });
        }
    }
};

/// Joins the binary names of a crate for its description.
fn joinBinaries(allocator: std.mem.Allocator, binaries: []const []const u8) ![]const u8 {
    var total: usize = 0;
    for (binaries) |binary| total += binary.len + 2;

    const buffer = try allocator.alloc(u8, total);
    errdefer allocator.free(buffer);
    var index: usize = 0;
    for (binaries, 0..) |binary, position| {
        if (position > 0) {
            buffer[index] = ',';
            buffer[index + 1] = ' ';
            index += 2;
        }
        @memcpy(buffer[index .. index + binary.len], binary);
        index += binary.len;
    }
    return buffer[0..index];
}

/// Reads the cargo index.
fn readIndex(allocator: std.mem.Allocator, io: std.Io, path: []const u8) ![]u8 {
    var file = std.Io.Dir.cwd().openFile(io, path, .{}) catch |err| switch (err) {
        error.FileNotFound => return error.FileNotFound,
        else => return error.IndexUnreadable,
    };
    defer file.close(io);

    var buffer: [8192]u8 = undefined;
    var reader = file.reader(io, &buffer);
    return reader.interface.allocRemaining(allocator, .unlimited) catch error.IndexUnreadable;
}

/// Parses the cargo index.
///
/// The index is a map from `<name> <version> (<source>)` to the crate's
/// metadata.
pub fn parseIndex(allocator: std.mem.Allocator, contents: []const u8) Error![]Crate {
    const parsed = std.json.parseFromSlice(std.json.Value, allocator, contents, .{}) catch
        return error.IndexMalformed;
    defer parsed.deinit();

    const root = parsed.value;
    if (root != .object) return error.IndexMalformed;
    const installs = root.object.get("installs") orelse return &.{};
    if (installs != .object) return error.IndexMalformed;

    var crates: std.ArrayList(Crate) = .empty;
    errdefer {
        for (crates.items) |*crate| crate.deinit(allocator);
        crates.deinit(allocator);
    }

    var iterator = installs.object.iterator();
    while (iterator.next()) |entry| {
        const key = entry.key_ptr.*;
        const parts = splitKey(key) orelse return error.IndexMalformed;

        // Everything the record keeps is copied out of the parsed document,
        // which is released when this function returns.
        const owned: Crate = blk: {
            const owned_name = try allocator.dupe(u8, parts.name);
            errdefer allocator.free(owned_name);
            const owned_version = try allocator.dupe(u8, parts.version);
            errdefer allocator.free(owned_version);
            const owned_source = try allocator.dupe(u8, parts.source);
            errdefer allocator.free(owned_source);

            var binaries: std.ArrayList([]const u8) = .empty;
            errdefer {
                for (binaries.items) |binary| allocator.free(binary);
                binaries.deinit(allocator);
            }
            if (entry.value_ptr.* == .object) {
                if (entry.value_ptr.*.object.get("bins")) |bins| {
                    if (bins == .array) {
                        for (bins.array.items) |item| {
                            if (item != .string) continue;
                            try binaries.append(allocator, try allocator.dupe(u8, item.string));
                        }
                    }
                }
            }

            break :blk .{
                .name = owned_name,
                .version = owned_version,
                .source = owned_source,
                .binaries = try binaries.toOwnedSlice(allocator),
            };
        };

        try crates.append(allocator, owned);
    }

    return crates.toOwnedSlice(allocator);
}

const ParsedKey = struct {
    name: []const u8,
    version: []const u8,
    source: []const u8,
};

/// Splits `<name> <version> (<source>)` into its parts.
///
/// The parts are slices into the key: the caller copies what it keeps.
fn splitKey(key: []const u8) ?ParsedKey {
    const first_space = std.mem.indexOfScalar(u8, key, ' ') orelse return null;
    const name = key[0..first_space];

    const rest = key[first_space + 1 ..];
    const second_space = std.mem.indexOfScalar(u8, rest, ' ') orelse return null;
    const version = rest[0..second_space];

    var source = rest[second_space + 1 ..];
    if (source.len >= 2 and source[0] == '(' and source[source.len - 1] == ')') {
        source = source[1 .. source.len - 1];
    }

    return .{ .name = name, .version = version, .source = source };
}

const testing = std.testing;

test "parseIndex: reads the name, version and registry of every crate" {
    const contents =
        \\{"installs":{
        \\  "cargo-edit 0.13.11 (registry+https://github.com/rust-lang/crates.io-index)":{"bins":["cargo-add","cargo-rm"]},
        \\  "ripgrep 14.1.1 (registry+https://github.com/rust-lang/crates.io-index)":{"bins":["rg"]}
        \\}}
    ;
    const crates = try parseIndex(testing.allocator, contents);
    defer deinitSlice(testing.allocator, crates);

    try testing.expectEqual(@as(usize, 2), crates.len);
    try testing.expectEqualStrings("cargo-edit", crates[0].name);
    try testing.expectEqualStrings("0.13.11", crates[0].version);
    try testing.expectEqualStrings("registry+https://github.com/rust-lang/crates.io-index", crates[0].source);
    try testing.expectEqual(@as(usize, 2), crates[0].binaries.len);
    try testing.expectEqualStrings("cargo-add", crates[0].binaries[0]);
}

test "parseIndex: a crate without binaries is still a crate" {
    const contents =
        \\{"installs":{"some-crate 1.0.0 (crates.io)":{}}}
    ;
    const crates = try parseIndex(testing.allocator, contents);
    defer deinitSlice(testing.allocator, crates);
    try testing.expectEqual(@as(usize, 1), crates.len);
    try testing.expectEqual(@as(usize, 0), crates[0].binaries.len);
}

test "parseIndex: an index without installs is empty" {
    const crates = try parseIndex(testing.allocator, "{}");
    defer deinitSlice(testing.allocator, crates);
    try testing.expectEqual(@as(usize, 0), crates.len);
}

test "parseIndex: a malformed index is rejected" {
    try testing.expectError(error.IndexMalformed, parseIndex(testing.allocator, "not json"));
    try testing.expectError(error.IndexMalformed, parseIndex(testing.allocator, "[]"));
    try testing.expectError(error.IndexMalformed, parseIndex(testing.allocator, "{\"installs\":[]}"));
}
