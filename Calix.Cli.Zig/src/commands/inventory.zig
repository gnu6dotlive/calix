//! `calix inventory` — prints the unified inventory.
//!
//! The gathering lives in `inventory/service.zig`, because the daemon
//! serves the same document. This command is the standalone path: it works
//! with no daemon running, which is the point of a CLI.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const runtime = @import("../runtime/context.zig");
const parser = @import("../cli/parser.zig");
const spec = @import("../cli/spec.zig");
const service = @import("../inventory/service.zig");

const record = Zigalpm.inventory.record;

const command_path = "calix inventory standard";

/// Whether this command owns the invocation.
///
/// The dispatcher asks every command in order whether it owns the
/// invocation, so a command that answered for everything would replace
/// every command that comes after it in the list.
fn claims(invocation: *const parser.Invocation) bool {
    return std.mem.eql(u8, invocation.command.path, command_path);
}

pub fn dispatch(
    context: *runtime.RuntimeContext,
    invocation: *const parser.Invocation,
) !?u8 {
    if (!claims(invocation)) return null;

    var snapshot = try service.snapshot(context, .{});
    defer snapshot.deinit();

    const json = try snapshot.toJson();
    defer context.allocator.free(json);

    try context.stdout.writeAll(json);
    try context.stdout.writeAll("\n");
    return 0;
}

test "the inventory command claims only its own path" {
    const inventory_command: spec.Command = .{
        .path = command_path,
        .parentPath = "calix",
        .name = "inventory",
        .description = null,
        .hidden = false,
        .isBranch = false,
        .hasAction = true,
        .aliases = &.{},
        .arguments = &.{},
        .options = &.{},
    };
    const resolve_command: spec.Command = .{
        .path = "calix resolve standard",
        .parentPath = "calix resolve",
        .name = "standard",
        .description = null,
        .hidden = false,
        .isBranch = false,
        .hasAction = true,
        .aliases = &.{},
        .arguments = &.{},
        .options = &.{},
    };

    const inventory_invocation: parser.Invocation = .{
        .command = &inventory_command,
        .arguments = &.{},
        .positionals = &.{},
        .options = &.{},
        .globals = .{},
    };
    const resolve_invocation: parser.Invocation = .{
        .command = &resolve_command,
        .arguments = &.{},
        .positionals = &.{},
        .options = &.{},
        .globals = .{},
    };

    try std.testing.expect(claims(&inventory_invocation));
    try std.testing.expect(!claims(&resolve_invocation));
}

test "the printed document is the versioned snapshot" {
    // The document a consumer parses must carry the version it was written
    // for: an unversioned document cannot be refused, only misread.
    const snapshot = record.Snapshot{
        .generated_at = 0,
        .packages = &.{},
    };
    const json = try snapshot.toJson(std.testing.allocator);
    defer std.testing.allocator.free(json);
    try std.testing.expect(std.mem.indexOf(u8, json, "\"version\":1") != null);
}
