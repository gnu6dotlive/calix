use crate::render_lab::catalog::FixtureCatalog;
use crate::render_lab::diagnostics::DiagnosticEntry;
use crate::render_lab::state::{ClockMode, RenderLabState};
use crate::render_lab::RENDER_LAB_SCHEMA_VERSION;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderLabManifest {
    pub schema_version: u32,
    pub gui_running: bool,
    pub active_fixture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    pub backend: String,
    pub topology: String,
    pub motion: String,
    pub quality: String,
    pub style: String,
    pub clock_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clock_time: Option<f32>,
    pub catalog_count: usize,
    pub diagnostics: Vec<DiagnosticEntry>,
}

impl RenderLabManifest {
    pub fn from_state(state: &RenderLabState, gui_running: bool) -> Self {
        let (clock_mode, clock_time) = match state.clock {
            ClockMode::Realtime => ("realtime".to_string(), None),
            ClockMode::Frozen(t) => ("frozen".to_string(), Some(t)),
        };
        let active_def = state.active_fixture.as_ref().and_then(FixtureCatalog::find);

        Self {
            schema_version: RENDER_LAB_SCHEMA_VERSION,
            gui_running,
            active_fixture: state
                .active_fixture
                .as_ref()
                .map(|f| f.as_str().to_string()),
            seed: active_def.map(|d| d.seed),
            recipe: state
                .active_fixture
                .as_ref()
                .and_then(|f| crate::render_lab::recipe::RecipeCatalog::find(f.as_str()))
                .map(|r| r.id.to_string()),
            backend: crate::render_lab::ledger::STOCK_BACKEND_ID.to_string(),
            topology: state.active_topology.as_str().to_string(),
            motion: state.active_motion.as_str().to_string(),
            quality: state.active_quality.as_str().to_string(),
            style: state.active_style.as_str().to_string(),
            clock_mode,
            clock_time,
            catalog_count: FixtureCatalog::count(),
            diagnostics: state.diagnostics.clone(),
        }
    }

    pub fn offline() -> Self {
        Self {
            schema_version: RENDER_LAB_SCHEMA_VERSION,
            gui_running: false,
            active_fixture: None,
            seed: None,
            recipe: None,
            backend: crate::render_lab::ledger::STOCK_BACKEND_ID.to_string(),
            topology: "floating-island".to_string(),
            motion: "classic".to_string(),
            quality: "stock".to_string(),
            style: crate::visual_style::VisualStyleId::Standard
                .as_str()
                .to_string(),
            clock_mode: "realtime".to_string(),
            clock_time: None,
            catalog_count: FixtureCatalog::count(),
            diagnostics: vec![DiagnosticEntry::info(
                "Calix GUI offline; static catalog manifest",
            )],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_lab_manifest_serde() {
        let manifest = RenderLabManifest::offline();
        assert_eq!(manifest.catalog_count, 46);
        assert!(!manifest.gui_running);

        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: RenderLabManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, manifest);
    }

    #[test]
    fn test_render_lab_manifest_from_state() {
        let mut state = RenderLabState::new();
        state.clock = ClockMode::Frozen(0.5);
        let manifest = RenderLabManifest::from_state(&state, true);
        assert!(manifest.gui_running);
        assert_eq!(manifest.clock_mode, "frozen");
        assert_eq!(manifest.clock_time, Some(0.5));
    }

    #[test]
    fn test_manifest_carries_confrontation_context() {
        let mut state = RenderLabState::new();
        state.active_fixture =
            Some(crate::render_lab::fixture::FixtureId::new("field.signed-voltage").unwrap());
        let manifest = RenderLabManifest::from_state(&state, true);
        assert_eq!(manifest.schema_version, 2);
        assert_eq!(
            manifest.active_fixture.as_deref(),
            Some("field.signed-voltage")
        );
        assert_eq!(manifest.seed, Some(1002));
        assert_eq!(manifest.recipe.as_deref(), Some("g02-bipolar-split/r1"));
        assert_eq!(manifest.backend, "gpui-stock-0.2.2");
        assert_eq!(manifest.style, "standard");

        let offline = RenderLabManifest::offline();
        assert_eq!(offline.seed, None);
        assert_eq!(offline.recipe, None);
        assert_eq!(offline.style, "standard");
    }
}
