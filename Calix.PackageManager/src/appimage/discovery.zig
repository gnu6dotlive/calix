//! Discovery of AppImages installed outside the package manager.
//!
//! The backend owns the AppImage database, but it only ever refreshed the
//! entries it already knew about (`syncAppImageMeta`). An AppImage dropped
//! into the installation directory by hand, or by another tool, was
//! invisible: the database only contained what the backend itself had
//! installed.
//!
//! A Python engine filled that gap by scanning the directory and writing
//! into the database — two writers on one file, with the Python treating a
//! file the backend owned as its own output. This module makes discovery a
//! backend responsibility: the backend reads its own directory, correlates
//! the XDG desktop entries, and commits its own database.
//!
//! The merge rule is the important part. Discovery *adds* what it finds and
//! *refreshes* what it can observe, but it never discards the fields the
//! backend owns and the filesystem cannot supply: the update URL and its
//! type, the forge coordinates, the prerelease opt-in, and the launch
//! environment. Those are configured deliberately; a directory scan must not
//! erase them.

const std = @import("std");
const appimage = @import("bindings.zig").appimage;
const launch_environment = @import("environment.zig");

/// The suffix that marks an executable as an AppImage.
pub const appimage_suffix = ".AppImage";

/// Desktop entry keys that may carry an AppImage version, in priority order.
const version_keys = [_][]const u8{
    "X-AppImage-Version",
    "X-OpenWork-Version",
    "X-Craft-Version",
    "X-ArtCraft-Version",
};

/// Where discovery looks, and what it correlates against.
pub const Options = struct {
    /// Directory holding the installed AppImages, e.g. `~/Applications`.
    install_directory: []const u8,
    /// Directory holding the XDG desktop entries, e.g.
    /// `~/.local/share/applications`.
    applications_directory: []const u8,
};

/// A desktop entry that launches an AppImage.
pub const DesktopEntry = struct {
    /// The AppImage path the entry executes.
    exec_path: []const u8,
    name: []const u8 = "",
    comment: []const u8 = "",
    icon: []const u8 = "",
    version: []const u8 = "",

    pub fn deinit(self: DesktopEntry, allocator: std.mem.Allocator) void {
        allocator.free(self.exec_path);
        allocator.free(self.name);
        allocator.free(self.comment);
        allocator.free(self.icon);
        allocator.free(self.version);
    }
};

/// Reads the XDG desktop entries that launch an AppImage.
///
/// Entries whose `Exec=` does not point at an AppImage are ignored.
///
/// When several desktop entries launch the same AppImage, the last one in
/// sorted filename order wins — the same rule the Python engine applied, so
/// that migrating the discovery to this backend does not change what the
/// user sees.
pub fn readDesktopEntries(
    allocator: std.mem.Allocator,
    io: std.Io,
    applications_directory: []const u8,
) ![]DesktopEntry {
    var names: std.ArrayList([]u8) = .empty;
    defer {
        for (names.items) |name| allocator.free(name);
        names.deinit(allocator);
    }

    var directory = std.Io.Dir.cwd().openDir(io, applications_directory, .{ .iterate = true }) catch |err| switch (err) {
        error.FileNotFound, error.NotDir => return try allocator.alloc(DesktopEntry, 0),
        else => return err,
    };
    defer directory.close(io);

    var iterator = directory.iterate();
    while (iterator.next(io) catch return error.DirectoryReadFailed) |directory_entry| {
        if (directory_entry.kind == .directory) continue;
        if (!std.mem.endsWith(u8, directory_entry.name, ".desktop")) continue;
        try names.append(allocator, try allocator.dupe(u8, directory_entry.name));
    }
    std.mem.sort([]u8, names.items, {}, struct {
        fn lessThan(_: void, left: []u8, right: []u8) bool {
            return std.mem.lessThan(u8, left, right);
        }
    }.lessThan);

    var entries: std.ArrayList(DesktopEntry) = .empty;
    errdefer {
        for (entries.items) |entry| entry.deinit(allocator);
        entries.deinit(allocator);
    }

    for (names.items) |name| {
        const path = try std.fs.path.join(allocator, &.{ applications_directory, name });
        defer allocator.free(path);

        const contents = readTextFile(allocator, io, path) catch continue;
        defer allocator.free(contents);

        const entry = parseDesktopEntry(allocator, contents) catch continue;
        if (entry.exec_path.len == 0) {
            entry.deinit(allocator);
            continue;
        }
        if (!std.mem.endsWith(u8, entry.exec_path, appimage_suffix)) {
            entry.deinit(allocator);
            continue;
        }
        // The last entry for a given AppImage wins. The previous entry is
        // released first, then replaced: copying the new record in transfers
        // its ownership, so nothing is freed twice.
        var replaced = false;
        for (entries.items) |*existing| {
            if (!std.mem.eql(u8, existing.exec_path, entry.exec_path)) continue;
            existing.deinit(allocator);
            existing.* = entry;
            replaced = true;
            break;
        }
        if (!replaced) try entries.append(allocator, entry);
    }

    return try entries.toOwnedSlice(allocator);
}

/// Parses a desktop entry, returning an entry with an empty `exec_path` when
/// the file does not launch an AppImage.
fn parseDesktopEntry(allocator: std.mem.Allocator, contents: []const u8) !DesktopEntry {
    var name: ?[]const u8 = null;
    var comment: ?[]const u8 = null;
    var icon: ?[]const u8 = null;
    var version: ?[]const u8 = null;
    var exec_path: ?[]const u8 = null;
    errdefer {
        if (name) |v| allocator.free(v);
        if (comment) |v| allocator.free(v);
        if (icon) |v| allocator.free(v);
        if (version) |v| allocator.free(v);
        if (exec_path) |v| allocator.free(v);
    }

    var lines = std.mem.splitScalar(u8, contents, '\n');
    while (lines.next()) |raw_line| {
        const line = std.mem.trim(u8, raw_line, " \t\r");
        if (line.len == 0 or line[0] == '#') continue;

        if (name == null) {
            if (valueOf(line, "Name")) |value| {
                name = try allocator.dupe(u8, value);
                continue;
            }
        }
        if (comment == null) {
            if (valueOf(line, "Comment")) |value| {
                comment = try allocator.dupe(u8, value);
                continue;
            }
        }
        if (icon == null) {
            if (valueOf(line, "Icon")) |value| {
                icon = try allocator.dupe(u8, value);
                continue;
            }
        }
        if (version == null) {
            for (version_keys) |key| {
                if (valueOf(line, key)) |value| {
                    version = try allocator.dupe(u8, value);
                    break;
                }
            }
            if (version != null) continue;
        }
        if (exec_path == null) {
            if (valueOf(line, "Exec")) |value| {
                if (appImagePathOf(value)) |path| {
                    exec_path = try allocator.dupe(u8, path);
                }
                continue;
            }
        }
    }

    return .{
        .exec_path = exec_path orelse try allocator.dupe(u8, ""),
        .name = name orelse try allocator.dupe(u8, ""),
        .comment = comment orelse try allocator.dupe(u8, ""),
        .icon = icon orelse try allocator.dupe(u8, ""),
        .version = version orelse try allocator.dupe(u8, ""),
    };
}

/// Returns the value of a `Key=Value` line, or null when the key differs.
fn valueOf(line: []const u8, key: []const u8) ?[]const u8 {
    if (!std.mem.startsWith(u8, line, key)) return null;
    if (line.len <= key.len or line[key.len] != '=') return null;
    return std.mem.trim(u8, line[key.len + 1 ..], " \t\r");
}

/// Extracts the AppImage path from an `Exec=` value.
///
/// The value may be quoted and may carry field codes (`%F`, `%U`, …).
fn appImagePathOf(exec_value: []const u8) ?[]const u8 {
    var candidate = exec_value;
    if (candidate.len > 0 and candidate[0] == '"') {
        // Search *after* the opening quote: searching the whole string
        // would find the opening one again and produce an invalid slice.
        const closing = std.mem.indexOfScalar(u8, candidate[1..], '"') orelse return null;
        candidate = candidate[1 .. 1 + closing];
    } else {
        // An unquoted path ends at the first space.
        if (std.mem.indexOfScalar(u8, candidate, ' ')) |space| {
            candidate = candidate[0..space];
        }
    }
    if (candidate.len == 0) return null;
    if (!std.mem.endsWith(u8, candidate, appimage_suffix)) return null;
    return candidate;
}

/// Scans the installation directory for AppImages and builds their records.
pub fn discover(
    allocator: std.mem.Allocator,
    io: std.Io,
    options: Options,
) ![]appimage.AppImage {
    const desktop_entries = try readDesktopEntries(allocator, io, options.applications_directory);
    defer {
        for (desktop_entries) |entry| entry.deinit(allocator);
        allocator.free(desktop_entries);
    }

    var discovered: std.ArrayList(appimage.AppImage) = .empty;
    errdefer {
        for (discovered.items) |item| freeAppImage(allocator, item);
        discovered.deinit(allocator);
    }

    var directory = std.Io.Dir.cwd().openDir(io, options.install_directory, .{ .iterate = true }) catch |err| switch (err) {
        error.FileNotFound, error.NotDir => return try discovered.toOwnedSlice(allocator),
        else => return err,
    };
    defer directory.close(io);

    var iterator = directory.iterate();
    while (iterator.next(io) catch return error.DirectoryReadFailed) |directory_entry| {
        if (directory_entry.kind == .directory) continue;
        if (!std.mem.endsWith(u8, directory_entry.name, appimage_suffix)) continue;

        const stat = directory.statFile(io, directory_entry.name, .{}) catch continue;
        if (stat.kind == .directory) continue;

        const path = try std.fs.path.join(allocator, &.{ options.install_directory, directory_entry.name });
        errdefer allocator.free(path);

        const entry = try buildAppImage(allocator, desktop_entries, directory_entry.name, path, stat.size);
        errdefer freeAppImage(allocator, entry);
        try discovered.append(allocator, entry);
    }

    return try discovered.toOwnedSlice(allocator);
}

/// Builds the record for one discovered AppImage.
fn buildAppImage(
    allocator: std.mem.Allocator,
    desktop_entries: []const DesktopEntry,
    file_name: []const u8,
    path: []const u8,
    size_on_disk: u64,
) !appimage.AppImage {
    const desktop = findDesktopEntry(desktop_entries, path);

    var name: []const u8 = "";
    if (desktop) |entry| {
        if (entry.name.len > 0) name = entry.name;
    }
    if (name.len == 0) name = try nameFromFileName(allocator, file_name);
    errdefer allocator.free(name);

    var version: []const u8 = "";
    if (desktop) |entry| {
        if (entry.version.len > 0) version = entry.version;
    }
    if (version.len == 0) version = try versionFromFileName(allocator, file_name);
    errdefer allocator.free(version);

    var icon_name: []const u8 = "";
    if (desktop) |entry| {
        if (entry.icon.len > 0) icon_name = entry.icon;
    }
    if (icon_name.len == 0) {
        const lowered = try allocator.dupe(u8, name);
        for (lowered) |*character| {
            character.* = std.ascii.toLower(character.*);
        }
        icon_name = lowered;
    }
    errdefer allocator.free(icon_name);

    var description: []const u8 = "";
    if (desktop) |entry| {
        if (entry.comment.len > 0) description = entry.comment;
    }
    if (description.len == 0) {
        description = try std.fmt.allocPrint(allocator, "{s} application", .{name});
    }
    errdefer allocator.free(description);

    const release_tag = try std.fmt.allocPrint(allocator, "{s}", .{
        if (std.mem.startsWith(u8, version, "v")) version else try std.fmt.allocPrint(allocator, "v{s}", .{version}),
    });
    errdefer allocator.free(release_tag);

    const owned_name = try allocator.dupe(u8, name);
    errdefer allocator.free(owned_name);
    const owned_version = try allocator.dupe(u8, version);
    errdefer allocator.free(owned_version);
    const owned_icon = try allocator.dupe(u8, icon_name);
    errdefer allocator.free(owned_icon);
    const owned_description = try allocator.dupe(u8, description);
    errdefer allocator.free(owned_description);
    const owned_path = try allocator.dupe(u8, path);
    errdefer allocator.free(owned_path);
    // Every string is owned: the manager frees the record field by field,
    // and a string literal must never reach the allocator's free.
    const owned_raw_update_info = try allocator.dupe(u8, "");
    errdefer allocator.free(owned_raw_update_info);
    const owned_command_line_args = try allocator.dupe(u8, "");
    errdefer allocator.free(owned_command_line_args);
    const owned_update_url = try allocator.dupe(u8, "");
    errdefer allocator.free(owned_update_url);

    return .{
        .name = owned_name,
        .version = owned_version,
        .release_tag = release_tag,
        .raw_update_info = owned_raw_update_info,
        .icon_name = owned_icon,
        .description = owned_description,
        .desktop_name = owned_name,
        .size_on_disk = size_on_disk,
        // An empty but owned slice: the manager frees the environment with
        // the allocator, and a static empty slice must never be freed.
        .environment_variables = try allocator.dupe(launch_environment.Variable, &.{}),
        .command_line_args = owned_command_line_args,
        .path = owned_path,
        .update_url = owned_update_url,
        .update_type = .none,
        .repo_owner = null,
        .repo_name = null,
        .allow_prerelease = false,
    };
}

/// Finds the desktop entry that launches the given path.
fn findDesktopEntry(entries: []const DesktopEntry, path: []const u8) ?DesktopEntry {
    for (entries) |entry| {
        if (std.mem.eql(u8, entry.exec_path, path)) return entry;
    }
    return null;
}

/// Derives a display name from an AppImage filename.
///
/// `cadcraft-0.8.0-linux-x86_64.AppImage` becomes `cadcraft`; a filename
/// without a recognisable version part keeps its stem.
fn nameFromFileName(allocator: std.mem.Allocator, file_name: []const u8) ![]const u8 {
    const stem = std.mem.trimEnd(u8, file_name, appimage_suffix);
    if (stem.len == 0) return allocator.dupe(u8, file_name);

    // Cut at the first separator that precedes something version-like.
    var best: ?[]const u8 = null;
    var index: usize = 0;
    while (index < stem.len) {
        const rest = stem[index..];
        const separator = std.mem.indexOfAny(u8, rest, "-_") orelse break;
        const candidate_end = index + separator;
        const tail = stem[candidate_end + 1 ..];
        if (tail.len > 0 and std.ascii.isDigit(tail[0])) {
            best = stem[0..candidate_end];
            break;
        }
        index = candidate_end + 1;
    }
    const chosen = best orelse stem;
    if (chosen.len == 0) return allocator.dupe(u8, file_name);
    return allocator.dupe(u8, chosen);
}

/// Derives a version from an AppImage filename.
///
/// The version is the first *dotted* numeric run in the name. A run without
/// a dot is not a version: `x86_64` carries two numbers and neither of them
/// is one. A trailing token is kept when it looks like a prerelease marker
/// (`26.8.4-nightly.202608290119`) and dropped when it is a platform
/// (`0.8.0-linux-x86_64`).
fn versionFromFileName(allocator: std.mem.Allocator, file_name: []const u8) ![]const u8 {
    const stem = std.mem.trimEnd(u8, file_name, appimage_suffix);
    if (stem.len == 0) return allocator.dupe(u8, "1.0.0");

    var index: usize = 0;
    while (index < stem.len) {
        if (!std.ascii.isDigit(stem[index])) {
            index += 1;
            continue;
        }

        // The maximal run of digits and dots.
        var end = index;
        var dots: usize = 0;
        while (end < stem.len) : (end += 1) {
            const character = stem[end];
            if (std.ascii.isDigit(character)) continue;
            if (character == '.') {
                dots += 1;
                continue;
            }
            break;
        }
        if (dots == 0) {
            index = end;
            continue;
        }

        // Trim a trailing dot from the run.
        var run_end = end;
        while (run_end > index and stem[run_end - 1] == '.') run_end -= 1;

        // A following token is a prerelease marker unless it names a platform.
        var version_end = run_end;
        if (run_end < stem.len and stem[run_end] == '-') {
            const suffix_start = run_end + 1;
            var suffix_end = suffix_start;
            while (suffix_end < stem.len and stem[suffix_end] != '-') suffix_end += 1;
            const suffix = stem[suffix_start..suffix_end];
            if (suffix.len > 0 and !isPlatformToken(suffix)) {
                version_end = suffix_end;
            }
        }
        return allocator.dupe(u8, stem[index..version_end]);
    }
    return allocator.dupe(u8, "1.0.0");
}

/// True when a filename token names a platform rather than a prerelease.
fn isPlatformToken(token: []const u8) bool {
    const platforms = [_][]const u8{
        "linux",       "x86_64",   "x86-64", "amd64",  "aarch64", "arm64",
        "i386",        "i686",     "riscv64", "win32", "windows", "mac",
        "macos",       "freebsd", "openbsd", "netbsd", "universal",
    };
    for (platforms) |platform| {
        if (std.ascii.eqlIgnoreCase(token, platform)) return true;
    }
    return false;
}

/// Frees a record's owned strings.
///
/// Mirrors the manager's `freeAppImage`: every string, plus the launch
/// environment, is released. Public because the caller owns the records
/// the discovery returned.
pub fn freeAppImage(allocator: std.mem.Allocator, item: appimage.AppImage) void {
    allocator.free(item.name);
    allocator.free(item.version);
    if (item.release_tag) |tag| allocator.free(tag);
    allocator.free(item.raw_update_info);
    allocator.free(item.icon_name);
    allocator.free(item.description);
    allocator.free(item.desktop_name);
    launch_environment.free(allocator, item.environment_variables);
    allocator.free(item.command_line_args);
    allocator.free(item.path);
    allocator.free(item.update_url);
    if (item.repo_owner) |owner| allocator.free(owner);
    if (item.repo_name) |name| allocator.free(name);
}

/// Merges discovered records into the existing database content.
///
/// The merge never *degrades* a record. A scan observes less than the
/// backend already knows: the database may carry the metadata extracted
/// from the AppImage binary itself, with the real desktop entry name and
/// the upstream version, while a directory scan can only see the filename
/// and the desktop files. So a field the database already fills in is kept,
/// and the scan only fills in the blanks.
///
/// The fields the filesystem cannot supply at all — the update URL and its
/// type, the forge coordinates, the prerelease opt-in, the launch
/// environment — are carried over untouched.
///
/// The returned list is fully owned — every string and every environment
/// variable is a fresh allocation — so the caller can release it with the
/// manager's own `freeAppImages`. Neither input is consumed.
pub fn merge(
    allocator: std.mem.Allocator,
    existing: []const appimage.AppImage,
    discovered: []const appimage.AppImage,
) ![]appimage.AppImage {
    var merged: std.ArrayList(appimage.AppImage) = .empty;
    errdefer {
        for (merged.items) |item| freeAppImage(allocator, item);
        merged.deinit(allocator);
    }

    for (discovered) |candidate| {
        var entry = try cloneAppImage(allocator, candidate);
        errdefer freeAppImage(allocator, entry);
        if (findByPath(existing, candidate.path)) |previous| {
            // Keep what the database already knows; fill in the blanks.
            try keepWhenPresent(allocator, previous.name, &entry.name);
            try keepWhenPresent(allocator, previous.version, &entry.version);
            try keepWhenPresent(allocator, previous.icon_name, &entry.icon_name);
            try keepWhenPresent(allocator, previous.description, &entry.description);
            try keepWhenPresent(allocator, previous.desktop_name, &entry.desktop_name);
            // The configured fields survive a scan.
            allocator.free(entry.update_url);
            entry.update_url = try allocator.dupe(u8, previous.update_url);
            entry.update_type = previous.update_type;
            if (entry.repo_owner) |owner| allocator.free(owner);
            if (previous.repo_owner) |owner| {
                entry.repo_owner = try allocator.dupe(u8, owner);
            } else {
                entry.repo_owner = null;
            }
            if (entry.repo_name) |name| allocator.free(name);
            if (previous.repo_name) |name| {
                entry.repo_name = try allocator.dupe(u8, name);
            } else {
                entry.repo_name = null;
            }
            entry.allow_prerelease = previous.allow_prerelease;
            launch_environment.free(allocator, entry.environment_variables);
            entry.environment_variables = try launch_environment.clone(
                allocator,
                previous.environment_variables,
            );
            allocator.free(entry.command_line_args);
            entry.command_line_args = try allocator.dupe(u8, previous.command_line_args);
            allocator.free(entry.raw_update_info);
            entry.raw_update_info = try allocator.dupe(u8, previous.raw_update_info);
        }
        try merged.append(allocator, entry);
    }

    // Records the scan did not find are kept: they may point at an AppImage
    // installed outside the scanned directory.
    for (existing) |item| {
        if (findByPath(discovered, item.path) != null) continue;
        const copy = try cloneAppImage(allocator, item);
        try merged.append(allocator, copy);
    }

    return try merged.toOwnedSlice(allocator);
}

/// Keeps the database's value when it is populated, otherwise takes the
/// discovered one.
fn keepWhenPresent(
    allocator: std.mem.Allocator,
    known: []const u8,
    discovered: *[]const u8,
) !void {
    if (known.len == 0) return;
    allocator.free(discovered.*);
    discovered.* = try allocator.dupe(u8, known);
}

/// Deep-copies a record so the copy owns every one of its strings.
fn cloneAppImage(allocator: std.mem.Allocator, item: appimage.AppImage) !appimage.AppImage {
    errdefer allocator.free(item.name);
    errdefer allocator.free(item.version);
    errdefer if (item.release_tag) |tag| allocator.free(tag);
    errdefer allocator.free(item.raw_update_info);
    errdefer allocator.free(item.icon_name);
    errdefer allocator.free(item.description);
    errdefer allocator.free(item.desktop_name);
    errdefer allocator.free(item.command_line_args);
    errdefer allocator.free(item.path);
    errdefer allocator.free(item.update_url);
    errdefer if (item.repo_owner) |owner| allocator.free(owner);
    errdefer if (item.repo_name) |name| allocator.free(name);
    errdefer launch_environment.free(allocator, item.environment_variables);

    return .{
        .name = try allocator.dupe(u8, item.name),
        .version = try allocator.dupe(u8, item.version),
        .release_tag = if (item.release_tag) |tag| try allocator.dupe(u8, tag) else null,
        .raw_update_info = try allocator.dupe(u8, item.raw_update_info),
        .icon_name = try allocator.dupe(u8, item.icon_name),
        .description = try allocator.dupe(u8, item.description),
        .desktop_name = try allocator.dupe(u8, item.desktop_name),
        .size_on_disk = item.size_on_disk,
        .environment_variables = try launch_environment.clone(allocator, item.environment_variables),
        .command_line_args = try allocator.dupe(u8, item.command_line_args),
        .path = try allocator.dupe(u8, item.path),
        .update_url = try allocator.dupe(u8, item.update_url),
        .update_type = item.update_type,
        .repo_owner = if (item.repo_owner) |owner| try allocator.dupe(u8, owner) else null,
        .repo_name = if (item.repo_name) |name| try allocator.dupe(u8, name) else null,
        .allow_prerelease = item.allow_prerelease,
    };
}

/// Finds a record by its path.
fn findByPath(items: []const appimage.AppImage, path: []const u8) ?appimage.AppImage {
    for (items) |item| {
        if (std.mem.eql(u8, item.path, path)) return item;
    }
    return null;
}

fn readTextFile(allocator: std.mem.Allocator, io: std.Io, path: []const u8) ![]u8 {
    var file = try std.Io.Dir.cwd().openFile(io, path, .{});
    defer file.close(io);
    var buffer: [4096]u8 = undefined;
    var reader = file.reader(io, &buffer);
    return reader.interface.allocRemaining(allocator, .unlimited);
}

const testing = std.testing;

test "valueOf: reads a key and rejects a near miss" {
    try testing.expectEqualStrings("Calix", valueOf("Name=Calix", "Name").?);
    try testing.expectEqualStrings("", valueOf("Name=", "Name").?);
    try testing.expect(valueOf("NameX=Calix", "Name") == null);
    try testing.expect(valueOf("X-Name=Calix", "Name") == null);
}

test "appImagePathOf: handles quotes, field codes and rejects non AppImages" {
    try testing.expectEqualStrings(
        "/home/u/Applications/cadcraft-0.8.0-linux-x86_64.AppImage",
        appImagePathOf("\"/home/u/Applications/cadcraft-0.8.0-linux-x86_64.AppImage\" %F").?,
    );
    try testing.expectEqualStrings(
        "/home/u/Applications/cadcraft-0.8.0-linux-x86_64.AppImage",
        appImagePathOf("/home/u/Applications/cadcraft-0.8.0-linux-x86_64.AppImage %F").?,
    );
    try testing.expect(appImagePathOf("/usr/bin/other") == null);
    try testing.expect(appImagePathOf("") == null);
}

test "nameFromFileName: strips the version and platform parts" {
    const cadcraft = try nameFromFileName(testing.allocator, "cadcraft-0.8.0-linux-x86_64.AppImage");
    defer testing.allocator.free(cadcraft);
    try testing.expectEqualStrings("cadcraft", cadcraft);

    const wordcraft = try nameFromFileName(testing.allocator, "wordcraft-0.8.0-linux-x86_64.AppImage");
    defer testing.allocator.free(wordcraft);
    try testing.expectEqualStrings("wordcraft", wordcraft);

    // A filename with no version keeps its stem: the platform suffix alone
    // is not a version boundary. The desktop entry supplies the real name.
    const copilot = try nameFromFileName(testing.allocator, "GitHub-Copilot-linux-x64.AppImage");
    defer testing.allocator.free(copilot);
    try testing.expectEqualStrings("GitHub-Copilot-linux-x64", copilot);

    // A platform token is not a version boundary, and neither is a number
    // that is not dotted: `x86_64` must not be read as the version 86.
    const openwork = try versionFromFileName(testing.allocator, "openwork-linux-x86_64-0.18.42.AppImage");
    defer testing.allocator.free(openwork);
    try testing.expectEqualStrings("0.18.42", openwork);
}

test "versionFromFileName: reads the version in both filename shapes" {
    const shapes = [_][2][]const u8{
        .{ "0.8.0", "cadcraft-0.8.0-linux-x86_64.AppImage" },
        .{ "0.18.42", "openwork-linux-x86_64-0.18.42.AppImage" },
        .{ "26.8.4-nightly.202608290119", "tty7-26.8.4-nightly.202608290119-linux-x86_64.AppImage" },
        .{ "1.1.82", "Netcatty-1.1.82-linux-x64.AppImage" },
    };
    for (shapes) |shape| {
        const version = try versionFromFileName(testing.allocator, shape[1]);
        defer testing.allocator.free(version);
        try testing.expectEqualStrings(shape[0], version);
    }
}

test "versionFromFileName: falls back when there is no version" {
    const version = try versionFromFileName(testing.allocator, "GitHub-Copilot-linux-x64.AppImage");
    defer testing.allocator.free(version);
    try testing.expectEqualStrings("1.0.0", version);
}

test "parseDesktopEntry: reads the metadata of an AppImage launcher" {
    const contents =
        \\[Desktop Entry]
        \\Type=Application
        \\Name=CADCraft
        \\Comment=Computer-aided design and drafting
        \\Exec="/home/u/Applications/cadcraft-0.8.0-linux-x86_64.AppImage" %F
        \\Icon=ai.storyteller.cadcraft
        \\X-AppImage-Version=0.8.0
    ;
    const entry = try parseDesktopEntry(testing.allocator, contents);
    defer entry.deinit(testing.allocator);
    try testing.expectEqualStrings("CADCraft", entry.name);
    try testing.expectEqualStrings("Computer-aided design and drafting", entry.comment);
    try testing.expectEqualStrings("ai.storyteller.cadcraft", entry.icon);
    try testing.expectEqualStrings("0.8.0", entry.version);
    try testing.expectEqualStrings("/home/u/Applications/cadcraft-0.8.0-linux-x86_64.AppImage", entry.exec_path);
}

test "merge: a discovered record keeps the configured update fields" {
    const previous = appimage.AppImage{
        .name = try testing.allocator.dupe(u8, "CADCraft"),
        .version = try testing.allocator.dupe(u8, "0.8.0"),
        .release_tag = try testing.allocator.dupe(u8, "v0.8.0"),
        .raw_update_info = try testing.allocator.dupe(u8, ""),
        .icon_name = try testing.allocator.dupe(u8, "ai.storyteller.cadcraft"),
        .description = try testing.allocator.dupe(u8, "d"),
        .desktop_name = try testing.allocator.dupe(u8, "CADCraft"),
        .size_on_disk = 10,
        .environment_variables = try testing.allocator.dupe(launch_environment.Variable, &.{}),
        .command_line_args = try testing.allocator.dupe(u8, ""),
        .path = try testing.allocator.dupe(u8, "/apps/CADCraft.AppImage"),
        .update_url = try testing.allocator.dupe(u8, "https://github.com/owner/repo"),
        .update_type = .github,
        .repo_owner = try testing.allocator.dupe(u8, "owner"),
        .repo_name = try testing.allocator.dupe(u8, "repo"),
        .allow_prerelease = true,
    };
    defer freeAppImage(testing.allocator, previous);

    const discovered = appimage.AppImage{
        .name = try testing.allocator.dupe(u8, "CADCraft"),
        .version = try testing.allocator.dupe(u8, "0.9.0"),
        .release_tag = try testing.allocator.dupe(u8, "v0.9.0"),
        .raw_update_info = try testing.allocator.dupe(u8, ""),
        .icon_name = try testing.allocator.dupe(u8, "ai.storyteller.cadcraft"),
        .description = try testing.allocator.dupe(u8, "d"),
        .desktop_name = try testing.allocator.dupe(u8, "CADCraft"),
        .size_on_disk = 20,
        .environment_variables = try testing.allocator.dupe(launch_environment.Variable, &.{}),
        .command_line_args = try testing.allocator.dupe(u8, ""),
        .path = try testing.allocator.dupe(u8, "/apps/CADCraft.AppImage"),
        .update_url = try testing.allocator.dupe(u8, ""),
        .update_type = .none,
        .repo_owner = null,
        .repo_name = null,
        .allow_prerelease = false,
    };
    defer freeAppImage(testing.allocator, discovered);

    const merged = try merge(testing.allocator, &.{previous}, &.{discovered});
    defer {
        for (merged) |item| freeAppImage(testing.allocator, item);
        testing.allocator.free(merged);
    }

    try testing.expectEqual(@as(usize, 1), merged.len);
    // The size is refreshed: it is what the filesystem observes.
    try testing.expectEqual(@as(u64, 20), merged[0].size_on_disk);
    // A populated field the database already knows is kept, not degraded.
    try testing.expectEqualStrings("0.8.0", merged[0].version);
    // The configured fields survive.
    try testing.expectEqualStrings("https://github.com/owner/repo", merged[0].update_url);
    try testing.expectEqual(appimage.UpdateType.github, merged[0].update_type);
    try testing.expectEqualStrings("owner", merged[0].repo_owner.?);
    try testing.expect(merged[0].allow_prerelease);
}

test "merge: a record outside the scanned directory is kept" {
    const elsewhere = appimage.AppImage{
        .name = try testing.allocator.dupe(u8, "Other"),
        .version = try testing.allocator.dupe(u8, "1.0.0"),
        .release_tag = null,
        .raw_update_info = try testing.allocator.dupe(u8, ""),
        .icon_name = try testing.allocator.dupe(u8, "other"),
        .description = try testing.allocator.dupe(u8, "d"),
        .desktop_name = try testing.allocator.dupe(u8, "Other"),
        .size_on_disk = 1,
        .environment_variables = try testing.allocator.dupe(launch_environment.Variable, &.{}),
        .command_line_args = try testing.allocator.dupe(u8, ""),
        .path = try testing.allocator.dupe(u8, "/opt/Other.AppImage"),
        .update_url = try testing.allocator.dupe(u8, ""),
        .update_type = .none,
        .repo_owner = null,
        .repo_name = null,
        .allow_prerelease = false,
    };
    defer freeAppImage(testing.allocator, elsewhere);

    const merged = try merge(testing.allocator, &.{elsewhere}, &.{});
    defer {
        for (merged) |item| freeAppImage(testing.allocator, item);
        testing.allocator.free(merged);
    }
    try testing.expectEqual(@as(usize, 1), merged.len);
    try testing.expectEqualStrings("/opt/Other.AppImage", merged[0].path);
}
