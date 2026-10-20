//! The unified inventory: every ecosystem the backend knows about, in one
//! versioned document.
//!
//! This is the document the daemon will serve and the frontend will
//! consume. It is produced here, in the CLI, because the CLI is where the
//! per-backend models live: the PackageManager owns the backends, the CLI
//! owns the composition.
//!
//! The shape is flat, not a tagged union. A union would make every new
//! ecosystem a breaking change for the consumer; a flat record with
//! optional fields lets a backend add detail nobody reads without
//! touching the others.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const runtime = @import("../runtime/context.zig");
const parser = @import("../cli/parser.zig");
const xdg = @import("../runtime/xdg.zig");
const aur_url = @import("../config/aur_url.zig");
const config_manager = @import("../config/manager.zig");
const config_model = @import("../config/model.zig");

const record = Zigalpm.inventory.record;

pub fn dispatch(
    context: *runtime.RuntimeContext,
    invocation: *const parser.Invocation,
) !?u8 {
    _ = invocation;

    var records: std.ArrayList(record.Record) = .empty;
    defer {
        for (records.items) |*item| item.deinit(context.allocator);
        records.deinit(context.allocator);
    }

    try appendAlpm(context, &records);
    try appendAur(context, &records);
    try appendFlatpak(context, &records);
    try appendAppImage(context, &records);
    try appendCargo(context, &records);
    try appendNpm(context, &records);

    const snapshot = record.Snapshot{
        .generated_at = @intCast(@divTrunc(std.Io.Timestamp.now(context.io, .awake).nanoseconds, std.time.ns_per_s)),
        .packages = records.items,
    };
    const json = try snapshot.toJson(context.allocator);
    defer context.allocator.free(json);

    try context.stdout.writeAll(json);
    try context.stdout.writeAll("\n");
    return 0;
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
