//! `shelly inventory` — prints the unified inventory.
//!
//! The gathering lives in `inventory/service.zig`, because the daemon
//! serves the same document. This command is the standalone path: it works
//! with no daemon running, which is the point of a CLI.

const std = @import("std");
const Zigalpm = @import("Zigalpm");

const runtime = @import("../runtime/context.zig");
const parser = @import("../cli/parser.zig");
const service = @import("../inventory/service.zig");

const record = Zigalpm.inventory.record;

pub fn dispatch(
    context: *runtime.RuntimeContext,
    invocation: *const parser.Invocation,
) !?u8 {
    _ = invocation;

    var snapshot = try service.snapshot(context, .{});
    defer snapshot.deinit();

    const json = try snapshot.toJson();
    defer context.allocator.free(json);

    try context.stdout.writeAll(json);
    try context.stdout.writeAll("\n");
    return 0;
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
