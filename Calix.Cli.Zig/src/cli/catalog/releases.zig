const types = @import("types.zig");

pub const variants = [_]types.Variant{
    .{
        .action = .releases,
        .name = "standard",
        .default_for_action = true,
        .description = "Show the release-tracked packages and what was last seen for each. The document is read from the state directory, so this works with the daemon stopped; --refresh asks the daemon to probe the tracked forges first.",
        .options = &.{types.flag(
            "--refresh",
            &.{},
            "Ask the daemon to probe the tracked releases now, then print what it committed",
        )},
    },
};
