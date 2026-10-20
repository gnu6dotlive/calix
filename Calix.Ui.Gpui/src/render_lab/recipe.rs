use serde::{Deserialize, Serialize};

/// Construction class of a confrontation attempt.
///
/// The kind describes HOW the recipe is built from stock primitives, never
/// the earned verdict: a `Native` attempt that misses fidelity still lands
/// `UNKNOWN` with a gap note. Verdicts live in the capability ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecipeKind {
    /// One stock primitive (single gradient, single shadow, single fill).
    Native,
    /// Several overlapping stock primitives, masks, or layout math.
    Composed,
    /// Immutable procedural memory texture through the proven `img()` path.
    Texture,
}

impl RecipeKind {
    pub const ALL: &[RecipeKind] = &[
        RecipeKind::Native,
        RecipeKind::Composed,
        RecipeKind::Texture,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Composed => "composed",
            Self::Texture => "texture",
        }
    }
}

/// One stock-backend confrontation recipe for a fixture.
///
/// `id` embeds the revision (`<family><cell>-<slug>/r<revision>`); any
/// recipe change mints a new revision so evidence never silently detaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeDef {
    pub id: &'static str,
    pub fixture: &'static str,
    pub kind: RecipeKind,
    pub revision: u32,
}

/// Static recipe table for all 46 canonical fixtures in RENDER-01.
pub struct RecipeCatalog;

static CANONICAL_RECIPES: &[RecipeDef] = &[
    // ── Group G: Fields (8 fixtures) ──────────────────────────────────
    RecipeDef {
        id: "g01-linear-scalar-gradient/r1",
        fixture: "field.zero-positive-scalar",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "g02-bipolar-split/r1",
        fixture: "field.signed-voltage",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "g03-magnitude-contour-stops/r1",
        fixture: "field.current-magnitude",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "g04-streamline-vector-field/r1",
        fixture: "field.current-direction",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "g05-thermal-multistop-gradient/r1",
        fixture: "field.overload-heat",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "g06-probabilistic-confidence-band/r1",
        fixture: "field.diagnostic-confidence",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "g07-stress-concentration-contour/r1",
        fixture: "field.component-stress",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "g08-continuous-density-plane/r1",
        fixture: "field.selection-density",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    // ── Group H: Depth (8 fixtures) ───────────────────────────────────
    RecipeDef {
        id: "h01-single-box-shadow/r1",
        fixture: "depth.contact-shadow",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "h02-diffused-elevation-shadow/r1",
        fixture: "depth.component-lift-shadow",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "h03-cavity-occlusion-bevel/r1",
        fixture: "depth.recessed-socket-shadow",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "h04-inset-panel-directional-bevel/r1",
        fixture: "depth.inset-panel-inner-shadow",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "h05-dual-tone-linear-bevel/r1",
        fixture: "depth.raised-instrument-subtle-bevel",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "h06-ambient-top-highlight-rim/r1",
        fixture: "depth.soft-edge-top-highlight",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "h07-continuous-perimeter-rim/r1",
        fixture: "depth.rim-highlight-outline",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "h08-prismatic-shallow-bevel/r1",
        fixture: "depth.shallow-bevel-3d-edge",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    // ── Group I: Optical (8 fixtures) ─────────────────────────────────
    RecipeDef {
        id: "i01-concentric-falloff/r1",
        fixture: "optical.radial-fade",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "i02-linear-directional-fade/r1",
        fixture: "optical.directional-fade",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "i03-soft-rounded-box-glow/r1",
        fixture: "optical.soft-rectangle-rounded",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "i04-radial-disc-glow-boundary/r1",
        fixture: "optical.soft-circle-radial",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "i05-corner-darkening-vignette/r1",
        fixture: "optical.edge-vignette",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "i06-central-luminance-boost/r1",
        fixture: "optical.local-focus",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "i07-filament-core-bloom/r1",
        fixture: "optical.energized-wire-glow",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "i08-thermal-emission-envelope/r1",
        fixture: "optical.heat-region-glow",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    // ── Group J: Materials (8 fixtures) ───────────────────────────────
    RecipeDef {
        id: "j01-matte-anthracite-coating/r1",
        fixture: "material.painted-metal.matte-anthracite",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "j02-signal-orange-coating/r1",
        fixture: "material.painted-metal.signal-orange",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "j03-beret-green-coating/r1",
        fixture: "material.painted-metal.beret-green",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "j04-royal-blue-coating/r1",
        fixture: "material.painted-metal.royal-blue",
        kind: RecipeKind::Native,
        revision: 1,
    },
    RecipeDef {
        id: "j05-brushed-sphere-texture/r1",
        fixture: "material.brushed-aluminum",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "j06-dark-anodized-satin/r1",
        fixture: "material.dark-anodized-aluminum",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    RecipeDef {
        id: "j07-warm-paper-fibrous-texture/r1",
        fixture: "material.warm-paper",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "j08-smoked-polycarbonate-translucent/r1",
        fixture: "material.smoked-plastic",
        kind: RecipeKind::Composed,
        revision: 1,
    },
    // ── Group K: Microstructure (14 fixtures) ─────────────────────────
    RecipeDef {
        id: "k01-uniform-noise-texture/r1",
        fixture: "micro.k01",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k02-stratified-jitter-texture/r1",
        fixture: "micro.k02",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k03-fine-grit-texture/r1",
        fixture: "micro.k03",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k04-brushed-micro-texture/r1",
        fixture: "micro.k04",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k05-anisotropic-grain-texture/r1",
        fixture: "micro.k05",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k06-cellular-pattern-texture/r1",
        fixture: "micro.k06",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k07-stochastic-stipple-texture/r1",
        fixture: "micro.k07",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k08-value-noise-lattice-texture/r1",
        fixture: "micro.k08",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k09-halftone-mesh-texture/r1",
        fixture: "micro.k09",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k10-woven-matrix-texture/r1",
        fixture: "micro.k10",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k11-crater-relief-texture/r1",
        fixture: "micro.k11",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k12-etched-fiber-texture/r1",
        fixture: "micro.k12",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k13-fine-dither-texture/r1",
        fixture: "micro.k13",
        kind: RecipeKind::Texture,
        revision: 1,
    },
    RecipeDef {
        id: "k14-coarse-grain-texture/r1",
        fixture: "micro.k14",
        kind: RecipeKind::Texture,
        revision: 1,
    },
];

impl RecipeCatalog {
    pub fn all() -> &'static [RecipeDef] {
        CANONICAL_RECIPES
    }

    pub fn count() -> usize {
        CANONICAL_RECIPES.len()
    }

    pub fn find(fixture_id: &str) -> Option<&'static RecipeDef> {
        CANONICAL_RECIPES.iter().find(|r| r.fixture == fixture_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use std::collections::HashSet;

    #[test]
    fn test_recipe_kinds_named() {
        assert_eq!(RecipeKind::ALL.len(), 3);
        assert_eq!(RecipeKind::Native.as_str(), "native");
        assert_eq!(RecipeKind::Composed.as_str(), "composed");
        assert_eq!(RecipeKind::Texture.as_str(), "texture");
    }

    #[test]
    fn test_canonical_catalog_shape() {
        assert_eq!(RecipeCatalog::count(), 46);
        let mut fixtures = HashSet::new();
        let mut ids = HashSet::new();
        for recipe in RecipeCatalog::all() {
            assert!(fixtures.insert(recipe.fixture), "duplicate fixture recipe");
            assert!(ids.insert(recipe.id), "duplicate recipe id");
            assert!(recipe.revision >= 1);
            assert!(recipe.id.ends_with(&format!("/r{}", recipe.revision)));
        }
        assert!(RecipeCatalog::find("field.zero-positive-scalar").is_some());
        assert!(RecipeCatalog::find("field.signed-voltage").is_some());
        assert!(RecipeCatalog::find("depth.contact-shadow").is_some());
        assert!(RecipeCatalog::find("optical.radial-fade").is_some());
        assert!(RecipeCatalog::find("material.brushed-aluminum").is_some());
        assert!(RecipeCatalog::find("micro.k01").is_some());
        assert!(RecipeCatalog::find("micro.k14").is_some());
    }

    #[test]
    fn test_canonical_covers_all_attempt_classes() {
        let kinds: HashSet<RecipeKind> = RecipeCatalog::all().iter().map(|r| r.kind).collect();
        assert_eq!(kinds.len(), 3);
        for kind in RecipeKind::ALL {
            assert!(kinds.contains(kind));
        }
    }
}
