//! The unified inventory service: every ecosystem the backend knows about,
//! gathered into one document.
//!
//! This is the implementation behind two consumers — `shelly inventory`,
//! which prints it, and `calix-daemon`, which serves it — so it lives
//! beside them rather than inside either. The PackageManager owns the
//! backends; this module owns the composition.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const runtime = @import("../runtime/context.zig");
const xdg = @import("../runtime/xdg.zig");
const aur_url = @import("../config/aur_url.zig");
const config_manager = @import("../config/manager.zig");
const config_model = @import("../config/model.zig");

pub const record = Zigalpm.inventory.record;
const tracked = Zigalpm.release.tracked;

/// What the gatherer needs to know that is not the machine it is looking at.
pub const Options = struct {
    /// Where the daemon keeps its state. The standalone CLI leaves this
    /// unset and gets the same directory the daemon would use, so both
    /// serve the same document.
    state_directory: ?[]const u8 = null,
};

/// A gathered inventory: the records and the time they were gathered.
pub const Snapshot = struct {
    allocator: std.mem.Allocator,
    records: []record.Record,
    generated_at: i64,

    pub fn deinit(self: *Snapshot) void {
        record.deinitSlice(self.allocator, self.records);
        self.* = undefined;
    }

    /// The document the frontend consumes.
    pub fn document(self: Snapshot) record.Snapshot {
        return .{
            .generated_at = self.generated_at,
            .packages = self.records,
        };
    }

    /// Serialises the document.
    pub fn toJson(self: Snapshot) ![]u8 {
        return self.document().toJson(self.allocator);
    }
};

/// Gathers every ecosystem into one snapshot.
///
/// A backend that is absent contributes nothing rather than failing the
/// whole gathering: Flatpak may not be installed, Cargo may never have
/// installed a crate, npm may not exist. The inventory is a view of what is
/// there, not a checklist of what should be.
pub fn snapshot(context: *runtime.RuntimeContext, options: Options) !Snapshot {
    var records: std.ArrayList(record.Record) = .empty;
    errdefer {
        for (records.items) |*item| item.deinit(context.allocator);
        records.deinit(context.allocator);
    }

    try appendAlpm(context, &records);
    try appendAur(context, &records);
    try appendFlatpak(context, &records);
    try appendAppImage(context, &records);
    try appendCargo(context, &records);
    try appendNpm(context, &records);
    try appendTracked(context, options, &records);

    return .{
        .allocator = context.allocator,
        .records = try records.toOwnedSlice(context.allocator),
        // `.real`, not `.awake`: `.awake` is monotonic since boot, and a
        // document that says it was generated in 1970 is worse than one
        // that omits the field.
        .generated_at = @intCast(@divTrunc(
            std.Io.Timestamp.now(context.io, .real).nanoseconds,
            std.time.ns_per_s,
        )),
    };
}

/// Appends the packages installed in the local ALPM database.
fn appendAlpm(context: *runtime.RuntimeContext, records: *std.ArrayList(record.Record)) !void {
    const manager = try Zigalpm.AlpmManager.init(
        context.allocator,
        context.environ,
        .{ .use_root = false },
    );
    defer manager.deinit();

    const native_items = try manager.get_installed_packages_with_reverse_dependencies(.{});
    defer Zigalpm.alpm.OwnedPackage.deinitSlice(context.allocator, native_items);

    for (native_items) |native| {
        const name = native.name() orelse continue;
        try records.append(context.allocator, .{
            .name = try context.allocator.dupe(u8, name),
            .version = try context.allocator.dupe(u8, native.version() orelse ""),
            .description = try context.allocator.dupe(u8, native.description() orelse ""),
            .source_type = .ALPM,
            .repository_or_remote = try context.allocator.dupe(u8, native.repository() orelse ""),
            .size_on_disk = @intCast(@max(0, native.install_size())),
        });
    }
}

/// Appends the foreign packages tracked as AUR packages.
fn appendAur(context: *runtime.RuntimeContext, records: *std.ArrayList(record.Record)) !void {
    const database_path = try xdg.shellyCache(context, &.{"db"});
    defer context.allocator.free(database_path);
    try std.Io.Dir.cwd().createDirPath(context.io, database_path);

    const aur_base = try aur_url.resolve(context, null);
    const manager = try Zigalpm.AurManager.init(context.allocator, context.environ, .{
        .aur_git_base_url = aur_base,
        .use_temp_path = true,
        .temp_path = database_path,
    });
    defer manager.deinit();

    const native_items = try manager.getInstalledPackagesWithReverseDependencies(.{});
    defer Zigalpm.aur.models.Package.deinitSlice(context.allocator, native_items);

    for (native_items) |native| {
        try records.append(context.allocator, .{
            .name = try context.allocator.dupe(u8, native.name),
            .version = try context.allocator.dupe(u8, native.version),
            .description = try context.allocator.dupe(u8, native.description orelse ""),
            .source_type = .AUR,
            .repository_or_remote = try context.allocator.dupe(u8, native.package_base),
        });
    }
}

/// Appends the installed Flatpak applications.
fn appendFlatpak(context: *runtime.RuntimeContext, records: *std.ArrayList(record.Record)) !void {
    var manager = Zigalpm.FlatpakManager{ .allocator = context.allocator, .io = context.io };
    defer manager.deinit();

    const native_items = try manager.list_installed_applications();
    defer Zigalpm.flatpak.manager.InstalledApplication.deinitSlice(context.allocator, native_items);

    for (native_items) |native| {
        try records.append(context.allocator, .{
            .name = try context.allocator.dupe(u8, native.name),
            .version = try context.allocator.dupe(u8, native.version),
            .description = try context.allocator.dupe(u8, native.summary),
            .source_type = .Flatpak,
            .repository_or_remote = try context.allocator.dupe(u8, native.origin),
            .size_on_disk = @intCast(@max(0, native.installed_size)),
        });
    }
}

/// Appends the AppImages the backend knows about.
fn appendAppImage(context: *runtime.RuntimeContext, records: *std.ArrayList(record.Record)) !void {
    const configuration = config_manager.Manager.init(context).read() catch
        try config_model.Config.defaults(context.allocator);
    const install_directory = stringValue(&configuration, "AppImageInstallPath") orelse
        try xdg.binHome(context);
    const local_db_path = try std.fs.path.join(
        context.allocator,
        &.{ try xdg.configHome(context), "shelly", "appimage-metadata-v2.db" },
    );
    defer context.allocator.free(local_db_path);

    var manager = Zigalpm.AppImageManager{
        .allocator = context.allocator,
        .io = context.io,
        .environ = context.environ,
        .install_directory = install_directory,
        .local_db_path = local_db_path,
    };
    defer manager.deinit();

    const native_items = try manager.getAppImagesFromLocalDb();
    defer manager.freeAppImages(native_items);

    for (native_items) |native| {
        try records.append(context.allocator, .{
            .name = try context.allocator.dupe(u8, native.name),
            .version = try context.allocator.dupe(u8, native.version),
            .description = try context.allocator.dupe(u8, native.description),
            .source_type = .AppImage,
            .repository_or_remote = try context.allocator.dupe(u8, native.desktop_name),
            .size_on_disk = native.size_on_disk,
            .icon_name = if (native.icon_name.len > 0)
                try context.allocator.dupe(u8, native.icon_name)
            else
                null,
            .path = if (native.path.len > 0)
                try context.allocator.dupe(u8, native.path)
            else
                null,
            .update_url = if (native.update_url.len > 0)
                try context.allocator.dupe(u8, native.update_url)
            else
                null,
            .repo_owner = if (native.repo_owner) |owner|
                try context.allocator.dupe(u8, owner)
            else
                null,
            .repo_name = if (native.repo_name) |name|
                try context.allocator.dupe(u8, name)
            else
                null,
            .allow_prerelease = native.allow_prerelease,
        });
    }
}

/// Appends the crates cargo has installed.
fn appendCargo(context: *runtime.RuntimeContext, records: *std.ArrayList(record.Record)) !void {
    const manager = Zigalpm.cargo.CargoManager.init(
        context.allocator,
        context.io,
        .{ .index_path = try cargoIndexPath(context) },
    );
    try manager.inventory(records);
}

/// Appends the globally installed npm packages.
fn appendNpm(context: *runtime.RuntimeContext, records: *std.ArrayList(record.Record)) !void {
    const manager = Zigalpm.npm.NpmManager.init(
        context.allocator,
        context.io,
        .{},
    );
    try manager.inventory(records);
}

/// Appends the packages a release is tracked for.
///
/// Transitional: see `release/tracked.zig`. The records exist so the
/// frontend keeps the row it has today while the file it used to read
/// disappears from the frontend.
fn appendTracked(
    context: *runtime.RuntimeContext,
    options: Options,
    records: *std.ArrayList(record.Record),
) !void {
    // A stack buffer, not an allocation: `xdg.stateHome` returns either the
    // environment's string or a fresh one, and there is no way to tell from
    // the outside which, so there is nothing safe to free.
    var buffer: [std.fs.max_path_bytes]u8 = undefined;
    const directory = options.state_directory orelse blk: {
        const home = try xdg.stateHome(context);
        break :blk try std.fmt.bufPrint(&buffer, "{s}/calix", .{home});
    };
    try tracked.append(context.allocator, context.io, directory, records);
}

/// The path of the cargo index, under the user's home directory.
fn cargoIndexPath(context: *runtime.RuntimeContext) ![]const u8 {
    const home = std.process.Environ.getPosix(context.environ, "HOME") orelse return error.HomeNotSet;
    return std.fs.path.join(context.allocator, &.{ home, Zigalpm.cargo.default_cargo_index_path });
}

/// Reads a string setting from the configuration.
fn stringValue(configuration: *const config_model.Config, key: []const u8) ?[]const u8 {
    const value = configuration.values.get(key) orelse return null;
    if (value != .string or value.string.len == 0) return null;
    return value.string;
}
