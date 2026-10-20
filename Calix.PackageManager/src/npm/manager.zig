//! The NPM ecosystem: globally installed packages.
//!
//! npm keeps no index of its own that can be read without running npm, so
//! the backend asks npm: `npm list -g --depth=0 --json`. That is the same
//! source of truth npm itself uses to uninstall a package, so a package the
//! backend reports is a package npm can remove.
//!
//! npm is not always installed, and the global prefix may be empty. Neither
//! is an error: the inventory simply has no NPM section.

const std = @import("std");
const record = @import("../inventory/record.zig");

pub const Error = error{
    /// npm could not be run.
    ToolFailed,
    /// npm produced output that could not be read.
    OutputUnreadable,
    /// npm produced output that is not the shape it documents.
    OutputMalformed,
    /// An allocation failed.
    OutOfMemory,
    /// npm is not installed.
    FileNotFound,
};

pub const Options = struct {
    /// The npm executable to run.
    program: []const u8 = "npm",
};

/// A globally installed npm package.
pub const Package = struct {
    name: []const u8,
    version: []const u8,
    /// Whether the installed version is overridden locally.
    overridden: bool,

    pub fn deinit(self: *Package, allocator: std.mem.Allocator) void {
        allocator.free(self.name);
        allocator.free(self.version);
        self.* = undefined;
    }
};

pub fn deinitSlice(allocator: std.mem.Allocator, packages: []Package) void {
    for (packages) |*package| package.deinit(allocator);
    allocator.free(packages);
}

pub const Manager = struct {
    allocator: std.mem.Allocator,
    io: std.Io,
    options: Options,

    pub fn init(allocator: std.mem.Allocator, io: std.Io, options: Options) Manager {
        return .{ .allocator = allocator, .io = io, .options = options };
    }

    /// Lists the globally installed npm packages.
    ///
    /// A missing npm is not an error: the inventory simply has no NPM
    /// section.
    pub fn list(self: Manager) Error![]Package {
        const output = self.runNpm() catch |err| switch (err) {
            error.FileNotFound => return &.{},
            else => return err,
        };
        defer self.allocator.free(output);

        return parseList(self.allocator, output);
    }

    /// Appends the packages as normalised inventory records.
    ///
    /// The records are appended into the caller's list, so ownership of
    /// every string is unambiguous.
    pub fn inventory(self: Manager, records: *std.ArrayList(record.Record)) Error!void {
        const packages = try self.list();
        defer deinitSlice(self.allocator, packages);

        for (packages) |package| {
            const description = if (package.overridden)
                "NPM global package (version overridden)"
            else
                "NPM global package";
            try records.append(self.allocator, .{
                .name = try self.allocator.dupe(u8, package.name),
                .version = try self.allocator.dupe(u8, package.version),
                .description = try self.allocator.dupe(u8, description),
                .source_type = .NPM,
                .repository_or_remote = try self.allocator.dupe(u8, "npm"),
                .overridden = package.overridden,
            });
        }
    }

    /// Runs `npm list -g --depth=0 --json` and returns its output.
    fn runNpm(self: Manager) Error![]u8 {
        const argv = [_][]const u8{ self.options.program, "list", "-g", "--depth=0", "--json" };
        var child = std.process.spawn(self.io, .{
            .argv = &argv,
            .stdin = .ignore,
            .stdout = .pipe,
            .stderr = .ignore,
        }) catch |err| switch (err) {
            // npm simply is not installed.
            error.FileNotFound => return error.FileNotFound,
            else => return error.ToolFailed,
        };
        defer child.kill(self.io);

        var read_buffer: [8192]u8 = undefined;
        var reader = child.stdout.?.reader(self.io, &read_buffer);
        const output = reader.interface.allocRemaining(self.allocator, .unlimited) catch
            return error.OutputUnreadable;
        const term = child.wait(self.io) catch return error.ToolFailed;
        // npm exits non-zero when a global package has an unmet dependency.
        // Its JSON is still the answer, so the exit status is not fatal.
        if (term == .exited and term.exited == 0) {} else if (output.len == 0) {
            return error.ToolFailed;
        }
        return output;
    }
};

/// Parses the output of `npm list -g --depth=0 --json`.
pub fn parseList(allocator: std.mem.Allocator, output: []const u8) Error![]Package {
    const parsed = std.json.parseFromSlice(std.json.Value, allocator, output, .{}) catch
        return error.OutputMalformed;
    defer parsed.deinit();

    const root = parsed.value;
    if (root != .object) return error.OutputMalformed;
    const dependencies = root.object.get("dependencies") orelse return &.{};
    if (dependencies != .object) return error.OutputMalformed;

    var packages: std.ArrayList(Package) = .empty;
    errdefer {
        for (packages.items) |*package| package.deinit(allocator);
        packages.deinit(allocator);
    }

    var iterator = dependencies.object.iterator();
    while (iterator.next()) |entry| {
        var version_text: []const u8 = "";
        var overridden = false;
        if (entry.value_ptr.* == .object) {
            if (entry.value_ptr.*.object.get("version")) |value| {
                if (value == .string) version_text = value.string;
            }
            if (entry.value_ptr.*.object.get("overridden")) |value| {
                if (value == .bool) overridden = value.bool;
            }
        }

        // Everything the record keeps is copied out of the parsed document,
        // which is released when this function returns.
        const owned: Package = blk: {
            const owned_name = try allocator.dupe(u8, entry.key_ptr.*);
            errdefer allocator.free(owned_name);
            const owned_version = try allocator.dupe(u8, version_text);
            errdefer allocator.free(owned_version);
            break :blk .{
                .name = owned_name,
                .version = owned_version,
                .overridden = overridden,
            };
        };

        try packages.append(allocator, owned);
    }

    return packages.toOwnedSlice(allocator);
}

const testing = std.testing;

test "parseList: reads every global package" {
    const output =
        \\{"name":"lib","version":"1.0.0","dependencies":{
        \\  "typescript":{"version":"5.6.3","overridden":false},
        \\  "local-override":{"version":"1.2.3","overridden":true}
        \\}}
    ;
    const packages = try parseList(testing.allocator, output);
    defer deinitSlice(testing.allocator, packages);

    try testing.expectEqual(@as(usize, 2), packages.len);
    try testing.expectEqualStrings("typescript", packages[0].name);
    try testing.expectEqualStrings("5.6.3", packages[0].version);
    try testing.expect(!packages[0].overridden);
    try testing.expect(packages[1].overridden);
}

test "parseList: a global prefix with no packages is empty" {
    const packages = try parseList(testing.allocator, "{\"dependencies\":{}}");
    defer deinitSlice(testing.allocator, packages);
    try testing.expectEqual(@as(usize, 0), packages.len);
}

test "parseList: output without dependencies is empty" {
    const packages = try parseList(testing.allocator, "{\"name\":\"lib\"}");
    defer deinitSlice(testing.allocator, packages);
    try testing.expectEqual(@as(usize, 0), packages.len);
}

test "parseList: malformed output is rejected" {
    try testing.expectError(error.OutputMalformed, parseList(testing.allocator, "not json"));
    try testing.expectError(error.OutputMalformed, parseList(testing.allocator, "[]"));
}
