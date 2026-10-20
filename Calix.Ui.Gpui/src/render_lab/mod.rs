pub mod capability;
pub mod catalog;
pub mod confront;
pub mod diagnostics;
pub mod experimental;
pub mod fixture;
pub mod graph;
pub mod ledger;
pub mod manifest;
pub mod plans;
pub mod recipe;
pub mod state;
pub mod texture;

pub use graph::{compile_stock_gpui, RecipeNode, RecipePlan, RecipeStructuralMetrics};
pub use plans::plan_for_recipe;

pub use capability::CapabilityClass;
pub use catalog::FixtureCatalog;
pub use diagnostics::{DiagnosticEntry, DiagnosticLevel};
pub use fixture::{
    FixtureDef, FixtureDeterminism, FixtureGroup, FixtureId, FixtureIdError, FixtureProvenance,
};
pub use ledger::{CapabilityLedger, CapabilityObservation, STOCK_BACKEND_ID};
pub use manifest::RenderLabManifest;
pub use state::{
    ClockMode, MotionVariant, QualityLevel, RenderLabEvent, RenderLabState, TopologyVariant,
};

pub const RENDER_LAB_SCHEMA_VERSION: u32 = 2;
