use crate::render_lab::graph::{BorderRadius, ColorSpec, Dim, RecipeNode, RecipePlan};
use crate::render_lab::texture::TextureKind;

/// Returns the canonical, deterministic `RecipePlan` for a given recipe ID.
pub fn plan_for_recipe(recipe_id: &str) -> Option<RecipePlan> {
    let root = match recipe_id {
        // ── Group G: Fields (8 fixtures) ──────────────────────────────
        "g01-linear-scalar-gradient/r1" => RecipeNode::rect()
            .size_full()
            .linear_gradient(
                90.0,
                ColorSpec::rgb(0x0e, 0x0f, 0x14),
                ColorSpec::rgb(0xfa, 0x6c, 0x00),
            )
            .build(),

        "g02-bipolar-split/r1" => RecipeNode::rect()
            .size_full()
            .flex_row()
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0x30, 0x57, 0xc7),
                        ColorSpec::rgb(0x9c, 0x9e, 0xaa),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0x9c, 0x9e, 0xaa),
                        ColorSpec::rgb(0xf8, 0x73, 0x19),
                    )
                    .build(),
            )
            .build(),

        "g03-magnitude-contour-stops/r1" => RecipeNode::rect()
            .size_full()
            .flex_row()
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0x13, 0x0e, 0x28),
                        ColorSpec::rgb(0x28, 0x1b, 0x54),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0x36, 0x27, 0x72),
                        ColorSpec::rgb(0x56, 0x38, 0xa2),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0x25, 0x6c, 0xb0),
                        ColorSpec::rgb(0x2c, 0xb3, 0xd4),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0x5a, 0xe4, 0xd8),
                        ColorSpec::rgb(0xe0, 0xff, 0xff),
                    )
                    .build(),
            )
            .build(),

        "g04-streamline-vector-field/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x12, 0x17, 0x1d))
            .center()
            .child(RecipeNode::streamlines(
                16,
                1.2,
                ColorSpec::hsla(196.0 / 360.0, 0.75, 0.65, 0.75),
            ))
            .build(),

        "g05-thermal-multistop-gradient/r1" => RecipeNode::rect()
            .size_full()
            .flex_row()
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0xff, 0xfe, 0xf5),
                        ColorSpec::rgb(0xfb, 0x92, 0x29),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0xfb, 0x92, 0x29),
                        ColorSpec::rgb(0xfb, 0x76, 0x29),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .w(Dim::Flex(1.0))
                    .h(Dim::Full)
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgb(0xfb, 0x76, 0x29),
                        ColorSpec::rgb(0xfa, 0x60, 0x2a),
                    )
                    .build(),
            )
            .build(),

        "g06-probabilistic-confidence-band/r1" => RecipeNode::rect()
            .size_full()
            .linear_gradient(
                90.0,
                ColorSpec::rgb(0x95, 0x93, 0x98),
                ColorSpec::rgb(0x62, 0x7e, 0x55),
            )
            .build(),

        "g07-stress-concentration-contour/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x10, 0x1b, 0x2b))
            .center()
            .child(
                RecipeNode::rect()
                    .size(Dim::Px(240.0))
                    .rounded(BorderRadius::Lg)
                    .solid(ColorSpec::rgb(0x18, 0x44, 0x6b))
                    .center()
                    .child(
                        RecipeNode::rect()
                            .size(Dim::Px(180.0))
                            .rounded(BorderRadius::Lg)
                            .solid(ColorSpec::rgb(0x28, 0x84, 0x6c))
                            .center()
                            .child(
                                RecipeNode::rect()
                                    .size(Dim::Px(120.0))
                                    .rounded(BorderRadius::Lg)
                                    .solid(ColorSpec::rgb(0xd4, 0xa0, 0x24))
                                    .center()
                                    .child(
                                        RecipeNode::rect()
                                            .size(Dim::Px(60.0))
                                            .rounded(BorderRadius::Lg)
                                            .solid(ColorSpec::rgb(0xd9, 0x2b, 0x2b))
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build(),

        "g08-continuous-density-plane/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0xa1, 0xa1, 0x9f))
            .relative()
            .child(
                RecipeNode::rect()
                    .size_full()
                    .linear_gradient(
                        90.0,
                        ColorSpec::rgba(0xff, 0xff, 0xff, 0x00),
                        ColorSpec::rgba(0x7c, 0x62, 0x91, 0x66),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .size_full()
                    .absolute_fill()
                    .linear_gradient(
                        180.0,
                        ColorSpec::rgba(0xff, 0xff, 0xff, 0x00),
                        ColorSpec::rgba(0xff, 0xff, 0xff, 0x88),
                    )
                    .build(),
            )
            .build(),

        // ── Group H: Depth (8 fixtures) ──────────────────────────────
        "h01-single-box-shadow/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0xf2, 0xf2, 0xf2))
            .child(
                RecipeNode::rect()
                    .size(Dim::Px(120.0))
                    .rounded(BorderRadius::Full)
                    .solid(ColorSpec::rgb(0xbd, 0xbd, 0xbd))
                    .shadow(0.0, 4.0, 3.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.60))
                    .build(),
            )
            .build(),

        "h02-diffused-elevation-shadow/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0xe8, 0xec, 0xef))
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(220.0))
                    .h(Dim::Px(140.0))
                    .rounded(BorderRadius::Xl)
                    .solid(ColorSpec::rgb(0xff, 0xff, 0xff))
                    .shadow(0.0, 14.0, 28.0, 2.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.22))
                    .build(),
            )
            .build(),

        "h03-cavity-occlusion-bevel/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0xdf, 0xe3, 0xe6))
            .child(
                RecipeNode::rect()
                    .size(Dim::Px(150.0))
                    .rounded(BorderRadius::Full)
                    .border_uniform(2.0, ColorSpec::rgba(0x00, 0x00, 0x00, 0x66))
                    .linear_gradient(
                        180.0,
                        ColorSpec::rgb(0x8a, 0x92, 0x99),
                        ColorSpec::rgb(0xad, 0xb5, 0xbd),
                    )
                    .center()
                    .child(
                        RecipeNode::rect()
                            .size(Dim::Px(110.0))
                            .rounded(BorderRadius::Full)
                            .solid(ColorSpec::rgb(0x2b, 0x30, 0x36))
                            .border_t(2.0, ColorSpec::rgba(0x00, 0x00, 0x00, 0xaa))
                            .build(),
                    )
                    .build(),
            )
            .build(),

        "h04-inset-panel-directional-bevel/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0xd4, 0xd8, 0xdc))
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(260.0))
                    .h(Dim::Px(160.0))
                    .rounded(BorderRadius::Lg)
                    .solid(ColorSpec::rgb(0xbc, 0xc2, 0xc8))
                    .border_t(2.0, ColorSpec::rgba(0x00, 0x00, 0x00, 0x77))
                    .border_b(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0xcc))
                    .child(
                        RecipeNode::rect()
                            .w(Dim::Full)
                            .h(Dim::Px(24.0))
                            .linear_gradient(
                                180.0,
                                ColorSpec::rgba(0x00, 0x00, 0x00, 0x55),
                                ColorSpec::rgba(0x00, 0x00, 0x00, 0x00),
                            )
                            .build(),
                    )
                    .build(),
            )
            .build(),

        "h05-dual-tone-linear-bevel/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0x28, 0x2c, 0x34))
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(220.0))
                    .h(Dim::Px(130.0))
                    .rounded(BorderRadius::Md)
                    .solid(ColorSpec::rgb(0x3a, 0x40, 0x4a))
                    .border_t(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0xaa))
                    .border_b(1.0, ColorSpec::rgba(0x00, 0x00, 0x00, 0x99))
                    .shadow(0.0, 6.0, 10.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.45))
                    .build(),
            )
            .build(),

        "h06-ambient-top-highlight-rim/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0x18, 0x1b, 0x20))
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(230.0))
                    .h(Dim::Px(140.0))
                    .rounded(BorderRadius::Lg)
                    .solid(ColorSpec::rgb(0x25, 0x2a, 0x32))
                    .border_t(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0x88))
                    .build(),
            )
            .build(),

        "h07-continuous-perimeter-rim/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0x16, 0x18, 0x1d))
            .child(
                RecipeNode::rect()
                    .size(Dim::Px(140.0))
                    .rounded(BorderRadius::Full)
                    .solid(ColorSpec::rgb(0x23, 0x27, 0x30))
                    .border_uniform(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0x77))
                    .build(),
            )
            .build(),

        "h08-prismatic-shallow-bevel/r1" => RecipeNode::rect()
            .size_full()
            .center()
            .solid(ColorSpec::rgb(0x1c, 0x20, 0x26))
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(220.0))
                    .h(Dim::Px(130.0))
                    .rounded(BorderRadius::Lg)
                    .padding(3.0)
                    .linear_gradient(
                        135.0,
                        ColorSpec::rgb(0x8a, 0x92, 0x9c),
                        ColorSpec::rgb(0x1b, 0x1d, 0x22),
                    )
                    .child(
                        RecipeNode::rect()
                            .size_full()
                            .rounded(BorderRadius::Md)
                            .linear_gradient(
                                180.0,
                                ColorSpec::rgb(0x35, 0x3a, 0x42),
                                ColorSpec::rgb(0x26, 0x29, 0x30),
                            )
                            .border_t(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0x55))
                            .build(),
                    )
                    .build(),
            )
            .build(),

        // ── Group I: Optical (8 fixtures) ────────────────────────────
        "i01-concentric-falloff/r1" => {
            const RINGS: usize = 32;
            const OUTER: f32 = 300.0;
            const INNER: f32 = 24.0;
            let mut nested: Option<RecipeNode> = None;
            for index in (0..RINGS).rev() {
                let t = index as f32 / (RINGS - 1) as f32;
                let diameter = INNER + (OUTER - INNER) * t;
                let alpha = (0.16 + 0.70 * (1.0 - t)) * 255.0;
                let ring = RecipeNode::rect()
                    .size(Dim::Px(diameter))
                    .rounded(BorderRadius::Full)
                    .solid(ColorSpec::rgba(0xff, 0xff, 0xff, alpha as u8))
                    .center();
                let ring_node = match nested {
                    Some(inner) => ring.child(inner).build(),
                    None => ring.build(),
                };
                nested = Some(ring_node);
            }
            RecipeNode::rect()
                .size_full()
                .center()
                .solid(ColorSpec::rgb(0xc9, 0xc9, 0xc9))
                .child(nested.expect("concentric falloff always builds rings"))
                .build()
        }

        "i02-linear-directional-fade/r1" => RecipeNode::rect()
            .size_full()
            .linear_gradient(
                90.0,
                ColorSpec::rgb(0x00, 0x99, 0xe6),
                ColorSpec::rgba(0x00, 0x99, 0xe6, 0x00),
            )
            .build(),

        "i03-soft-rounded-box-glow/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x0b, 0x13, 0x1c))
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(200.0))
                    .h(Dim::Px(120.0))
                    .rounded(BorderRadius::Xl)
                    .solid(ColorSpec::rgb(0x20, 0xa0, 0xd0))
                    .shadow(
                        0.0,
                        0.0,
                        34.0,
                        10.0,
                        ColorSpec::hsla(196.0 / 360.0, 0.85, 0.60, 0.65),
                    )
                    .build(),
            )
            .build(),

        "i04-radial-disc-glow-boundary/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x0c, 0x12, 0x19))
            .center()
            .child(
                RecipeNode::rect()
                    .size(Dim::Px(100.0))
                    .rounded(BorderRadius::Full)
                    .solid(ColorSpec::rgb(0x60, 0xd8, 0xff))
                    .shadow(
                        0.0,
                        0.0,
                        45.0,
                        25.0,
                        ColorSpec::hsla(195.0 / 360.0, 0.95, 0.55, 0.70),
                    )
                    .shadow(
                        0.0,
                        0.0,
                        15.0,
                        5.0,
                        ColorSpec::hsla(195.0 / 360.0, 0.90, 0.75, 0.80),
                    )
                    .build(),
            )
            .build(),

        "i05-corner-darkening-vignette/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0xc0, 0xc8, 0xd0))
            .justify_between_col()
            .child(
                RecipeNode::rect()
                    .w(Dim::Full)
                    .h(Dim::Px(60.0))
                    .linear_gradient(
                        180.0,
                        ColorSpec::rgba(0x00, 0x00, 0x00, 0x66),
                        ColorSpec::rgba(0x00, 0x00, 0x00, 0x00),
                    )
                    .build(),
            )
            .child(
                RecipeNode::rect()
                    .w(Dim::Full)
                    .h(Dim::Px(60.0))
                    .linear_gradient(
                        0.0,
                        ColorSpec::rgba(0x00, 0x00, 0x00, 0x66),
                        ColorSpec::rgba(0x00, 0x00, 0x00, 0x00),
                    )
                    .build(),
            )
            .build(),

        "i06-central-luminance-boost/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x0e, 0x14, 0x1c))
            .center()
            .child(
                RecipeNode::rect()
                    .size(Dim::Px(140.0))
                    .rounded(BorderRadius::Full)
                    .solid(ColorSpec::rgb(0xff, 0xff, 0xff))
                    .shadow(
                        0.0,
                        0.0,
                        60.0,
                        30.0,
                        ColorSpec::hsla(210.0 / 360.0, 0.80, 0.70, 0.55),
                    )
                    .build(),
            )
            .build(),

        "i07-filament-core-bloom/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x14, 0x0f, 0x0a))
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Full)
                    .h(Dim::Px(3.0))
                    .solid(ColorSpec::rgb(0xff, 0xf8, 0xd0))
                    .shadow(
                        0.0,
                        0.0,
                        12.0,
                        4.0,
                        ColorSpec::hsla(38.0 / 360.0, 1.0, 0.55, 0.90),
                    )
                    .shadow(
                        0.0,
                        0.0,
                        35.0,
                        12.0,
                        ColorSpec::hsla(24.0 / 360.0, 1.0, 0.45, 0.65),
                    )
                    .build(),
            )
            .build(),

        "i08-thermal-emission-envelope/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x12, 0x08, 0x08))
            .center()
            .child(
                RecipeNode::rect()
                    .size(Dim::Px(120.0))
                    .rounded(BorderRadius::Full)
                    .solid(ColorSpec::rgb(0xff, 0xaa, 0x20))
                    .shadow(
                        0.0,
                        0.0,
                        40.0,
                        20.0,
                        ColorSpec::hsla(16.0 / 360.0, 1.0, 0.50, 0.85),
                    )
                    .shadow(
                        0.0,
                        0.0,
                        80.0,
                        40.0,
                        ColorSpec::hsla(0.0 / 360.0, 1.0, 0.40, 0.55),
                    )
                    .build(),
            )
            .build(),

        // ── Group J: Materials (8 fixtures) ──────────────────────────
        "j01-matte-anthracite-coating/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x16, 0x18, 0x1b))
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(240.0))
                    .h(Dim::Px(150.0))
                    .rounded(BorderRadius::Lg)
                    .solid(ColorSpec::rgb(0x22, 0x25, 0x2a))
                    .border_uniform(1.0, ColorSpec::rgb(0x2e, 0x32, 0x38))
                    .border_t(1.0, ColorSpec::rgb(0x45, 0x4b, 0x54))
                    .shadow(0.0, 8.0, 16.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.50))
                    .build(),
            )
            .build(),

        "j02-signal-orange-coating/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x1c, 0x1e, 0x22))
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(240.0))
                    .h(Dim::Px(150.0))
                    .rounded(BorderRadius::Lg)
                    .linear_gradient(
                        180.0,
                        ColorSpec::rgb(0xff, 0x6e, 0x2b),
                        ColorSpec::rgb(0xdb, 0x4c, 0x0b),
                    )
                    .border_uniform(1.0, ColorSpec::rgb(0xb0, 0x38, 0x00))
                    .shadow(0.0, 8.0, 16.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.45))
                    .build(),
            )
            .build(),

        "j03-beret-green-coating/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x18, 0x1a, 0x1c))
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(240.0))
                    .h(Dim::Px(150.0))
                    .rounded(BorderRadius::Lg)
                    .linear_gradient(
                        180.0,
                        ColorSpec::rgb(0x45, 0x56, 0x48),
                        ColorSpec::rgb(0x31, 0x3d, 0x33),
                    )
                    .border_uniform(1.0, ColorSpec::rgb(0x23, 0x2c, 0x25))
                    .shadow(0.0, 8.0, 16.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.45))
                    .build(),
            )
            .build(),

        "j04-royal-blue-coating/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x15, 0x18, 0x1d))
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(240.0))
                    .h(Dim::Px(150.0))
                    .rounded(BorderRadius::Lg)
                    .linear_gradient(
                        180.0,
                        ColorSpec::rgb(0x25, 0x5b, 0xb5),
                        ColorSpec::rgb(0x14, 0x3c, 0x82),
                    )
                    .border_uniform(1.0, ColorSpec::rgb(0x0e, 0x29, 0x59))
                    .shadow(0.0, 8.0, 16.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.50))
                    .build(),
            )
            .build(),

        "j05-brushed-sphere-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::BrushedCell, 4005, 256, 256)
            .build(),

        "j06-dark-anodized-satin/r1" => RecipeNode::rect()
            .size_full()
            .solid(ColorSpec::rgb(0x18, 0x1a, 0x1d))
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(240.0))
                    .h(Dim::Px(150.0))
                    .rounded(BorderRadius::Lg)
                    .linear_gradient(
                        180.0,
                        ColorSpec::rgb(0x35, 0x3a, 0x40),
                        ColorSpec::rgb(0x27, 0x2b, 0x30),
                    )
                    .border_uniform(1.0, ColorSpec::rgb(0x40, 0x45, 0x4c))
                    .border_t(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0x55))
                    .shadow(0.0, 6.0, 14.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.55))
                    .build(),
            )
            .build(),

        "j07-warm-paper-fibrous-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::WarmPaper, 4007, 256, 256)
            .build(),

        "j08-smoked-polycarbonate-translucent/r1" => RecipeNode::rect()
            .size_full()
            .linear_gradient(
                45.0,
                ColorSpec::rgb(0x2c, 0x3e, 0x50),
                ColorSpec::rgb(0xbd, 0xc3, 0xc7),
            )
            .center()
            .child(
                RecipeNode::rect()
                    .w(Dim::Px(240.0))
                    .h(Dim::Px(150.0))
                    .rounded(BorderRadius::Xl)
                    .solid(ColorSpec::rgba(0x14, 0x18, 0x1e, 0xcc))
                    .border_uniform(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0x44))
                    .border_t(1.0, ColorSpec::rgba(0xff, 0xff, 0xff, 0xaa))
                    .shadow(0.0, 12.0, 24.0, 0.0, ColorSpec::hsla(0.0, 0.0, 0.0, 0.40))
                    .build(),
            )
            .build(),

        // ── Group K: Microstructures (14 fixtures) ───────────────────
        "k01-uniform-noise-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::UniformNoise, 5001, 256, 256)
            .build(),

        "k02-stratified-jitter-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::StratifiedJitter, 5002, 256, 256)
            .build(),

        "k03-fine-grit-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::FineGrit, 5003, 256, 256)
            .build(),

        "k04-brushed-micro-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::BrushedMicro, 5004, 256, 256)
            .build(),

        "k05-anisotropic-grain-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::AnisotropicGrain, 5005, 256, 256)
            .build(),

        "k06-cellular-pattern-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::Cellular, 5006, 256, 256)
            .build(),

        "k07-stochastic-stipple-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::Stipple, 5007, 256, 256)
            .build(),

        "k08-value-noise-lattice-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::ValueNoiseLattice, 5008, 256, 256)
            .build(),

        "k09-halftone-mesh-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::HalftoneMesh, 5009, 256, 256)
            .build(),

        "k10-woven-matrix-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::WovenMatrix, 5010, 256, 256)
            .build(),

        "k11-crater-relief-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::CraterRelief, 5011, 256, 256)
            .build(),

        "k12-etched-fiber-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::EtchedFiber, 5012, 256, 256)
            .build(),

        "k13-fine-dither-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::FineDither, 5013, 256, 256)
            .build(),

        "k14-coarse-grain-texture/r1" => RecipeNode::rect()
            .size_full()
            .texture(TextureKind::CoarseGrain, 5014, 256, 256)
            .build(),

        _ => return None,
    };

    Some(RecipePlan::new(recipe_id, root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_lab::recipe::RecipeCatalog;
    use core::prelude::v1::test;

    #[test]
    fn test_all_46_canonical_recipes_have_plans() {
        let recipes = RecipeCatalog::all();
        assert_eq!(recipes.len(), 46);
        for recipe in recipes {
            let plan = plan_for_recipe(recipe.id);
            assert!(
                plan.is_some(),
                "recipe {} must have a RecipePlan",
                recipe.id
            );
            let p = plan.unwrap();
            let metrics = p.structural_metrics();
            assert!(
                metrics.graph_nodes >= 1,
                "recipe {} has no graph nodes",
                recipe.id
            );
            assert!(
                metrics.expanded_ops >= 1,
                "recipe {} has no operations",
                recipe.id
            );
        }
    }

    #[test]
    fn test_all_46_plans_serialization_lossless() {
        let recipes = RecipeCatalog::all();
        for recipe in recipes {
            let plan = plan_for_recipe(recipe.id).expect("plan must exist");
            let serialized = serde_json::to_string(&plan).expect("plan serializes");
            let deserialized: RecipePlan =
                serde_json::from_str(&serialized).expect("plan deserializes");
            assert_eq!(plan, deserialized, "round-trip mismatch for {}", recipe.id);
            assert_eq!(
                plan.structural_metrics(),
                deserialized.structural_metrics(),
                "metrics mismatch after deserialization for {}",
                recipe.id
            );
        }
    }

    #[test]
    fn test_texture_structural_metrics_exact() {
        let texture_recipes = [
            "j05-brushed-sphere-texture/r1",
            "j07-warm-paper-fibrous-texture/r1",
            "k01-uniform-noise-texture/r1",
            "k02-stratified-jitter-texture/r1",
            "k03-fine-grit-texture/r1",
            "k04-brushed-micro-texture/r1",
            "k05-anisotropic-grain-texture/r1",
            "k06-cellular-pattern-texture/r1",
            "k07-stochastic-stipple-texture/r1",
            "k08-value-noise-lattice-texture/r1",
            "k09-halftone-mesh-texture/r1",
            "k10-woven-matrix-texture/r1",
            "k11-crater-relief-texture/r1",
            "k12-etched-fiber-texture/r1",
            "k13-fine-dither-texture/r1",
            "k14-coarse-grain-texture/r1",
        ];

        assert_eq!(texture_recipes.len(), 16);
        for id in texture_recipes {
            let plan = plan_for_recipe(id).expect("texture recipe plan must exist");
            let m = plan.structural_metrics();
            assert_eq!(m.texture_count, 1, "expected 1 texture for {id}");
            assert_eq!(m.texture_pixels, 256 * 256, "expected 65536 px for {id}");
            assert_eq!(
                m.texture_rgba_bytes,
                256 * 256 * 4,
                "expected 262144 bytes for {id}"
            );
        }
    }
}
