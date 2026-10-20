const types = @import("types.zig");

pub const variants = [_]types.Variant{
    .{
        .action = .inventory,
        .name = "standard",
        .default_for_action = true,
        .description = "Print the unified inventory of every ecosystem — ALPM, AUR, Flatpak, AppImage, Cargo and NPM — as one versioned JSON document.",
        .implementation = "Zigalpm.inventory.record.Snapshot",
    },
};
