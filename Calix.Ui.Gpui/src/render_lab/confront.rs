use crate::render_lab::fixture::{FixtureDef, FixtureGroup};
use crate::render_lab::graph::compile_stock_gpui;
use crate::render_lab::plans::plan_for_recipe;
use crate::render_lab::recipe::{RecipeCatalog, RecipeDef};
use gpui::AnyElement;

/// Fixed preview-box height: the confrontation surface is always 340px
/// tall, full column width. ROI width is session-dependent and recorded
/// per capture; ROI height is structurally constant.
pub const CONFRONT_BOX_HEIGHT_PX: f32 = 340.0;

/// Fiducial marker color framing the confrontation box. The determinism
/// harness locates this exact color in window captures to derive the
/// fixed preview ROI; the marker itself is excluded from pixel hashing.
pub const FIDUCIAL_MARKER_RGB: u32 = 0xFF00FF;

/// Exact atlas-backed canvas for the first direct-spatial comparison slice.
/// The fiducial wrapper is four pixels larger so the existing two-pixel inset
/// extracts the content rectangle without a fallback resize.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComparisonCanvasSpec {
    pub width_px: u32,
    pub height_px: u32,
}

pub fn comparison_canvas_spec(def: &FixtureDef) -> Option<ComparisonCanvasSpec> {
    (def.group == FixtureGroup::Fields).then_some(ComparisonCanvasSpec {
        width_px: 427,
        height_px: 87,
    })
}

/// Renders the stock confrontation for a fixture, or `None` when no
/// recipe exists yet (unevaluated slot).
/// In RENDER-02, this dispatches via `plan_for_recipe` through the
/// single deterministic `compile_stock_gpui` compiler.
pub fn render_confrontation(def: &FixtureDef) -> Option<AnyElement> {
    let recipe = RecipeCatalog::find(def.id.as_str())?;
    let plan = plan_for_recipe(recipe.id)?;
    Some(compile_stock_gpui(&plan))
}

pub fn recipe_for(def: &FixtureDef) -> Option<&'static RecipeDef> {
    RecipeCatalog::find(def.id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confront_box_geometry_constants() {
        assert_eq!(CONFRONT_BOX_HEIGHT_PX, 340.0);
        assert_eq!(FIDUCIAL_MARKER_RGB, 0xFF00FF);
    }

    #[test]
    fn test_fields_use_atlas_canvas_dimensions() {
        for def in crate::render_lab::catalog::FixtureCatalog::for_group(FixtureGroup::Fields) {
            assert_eq!(
                comparison_canvas_spec(def),
                Some(ComparisonCanvasSpec {
                    width_px: 427,
                    height_px: 87,
                })
            );
        }
    }

    #[test]
    fn test_all_46_fixtures_have_recipes_and_confrontations() {
        let catalog = crate::render_lab::catalog::FixtureCatalog::all();
        assert_eq!(catalog.len(), 46);
        for def in catalog {
            assert!(
                recipe_for(def).is_some(),
                "missing recipe for fixture {}",
                def.id.as_str()
            );
            assert!(
                render_confrontation(def).is_some(),
                "missing confrontation renderer for fixture {}",
                def.id.as_str()
            );
        }
    }
}
