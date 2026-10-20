use crate::render_lab::capability::CapabilityClass;
use crate::render_lab::fixture::{
    FixtureDef, FixtureDeterminism, FixtureGroup, FixtureId, FixtureProvenance,
};
use std::sync::OnceLock;

static FIXTURE_CATALOG: OnceLock<Vec<FixtureDef>> = OnceLock::new();

pub const GHIJ_SOURCE_ARTIFACT: &str = "gnosix-board-GHIJ-source.png";
pub const GHIJ_SOURCE_SHA256: &str =
    "96d94a6b1cd7c5680c5e1432d49acbb92be0873d60eff34408c9ffbb5675ac16";

pub const K_SOURCE_ARTIFACT: &str = "gnosix-board-K-source.png";
pub const K_SOURCE_SHA256: &str =
    "950682c3fa7ef9e51d1b8604c1f15d25d5fe1e446b2b47c9cc9120387b8726b8";

pub struct FixtureCatalog;

impl FixtureCatalog {
    pub fn all() -> &'static [FixtureDef] {
        FIXTURE_CATALOG.get_or_init(build_catalog).as_slice()
    }

    pub fn find(id: &FixtureId) -> Option<&'static FixtureDef> {
        Self::all().iter().find(|f| &f.id == id)
    }

    pub fn find_by_str(id_str: &str) -> Option<&'static FixtureDef> {
        Self::all().iter().find(|f| f.id.as_str() == id_str)
    }

    pub fn count() -> usize {
        Self::all().len()
    }

    pub fn for_group(group: FixtureGroup) -> impl Iterator<Item = &'static FixtureDef> {
        Self::all().iter().filter(move |f| f.group == group)
    }
}

fn build_catalog() -> Vec<FixtureDef> {
    vec![
        // ── Group G: Fields (8 fixtures) ──────────────────────────────────
        FixtureDef {
            id: FixtureId::from_static("field.zero-positive-scalar"),
            group: FixtureGroup::Fields,
            seed: 1001,
            description: "Uniform non-negative scalar field baseline",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 1,
                source_label: Some("Zero-to-positive scalar"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("field.signed-voltage"),
            group: FixtureGroup::Fields,
            seed: 1002,
            description: "Bipolar signed scalar field with zero-crossing threshold",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 2,
                source_label: Some("Signed voltage"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("field.current-magnitude"),
            group: FixtureGroup::Fields,
            seed: 1003,
            description: "Scalar magnitude representation of current flow",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 3,
                source_label: Some("Current magnitude"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("field.current-direction"),
            group: FixtureGroup::Fields,
            seed: 1004,
            description: "Directional vector orientation indicator",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 4,
                source_label: Some("Current direction"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("field.overload-heat"),
            group: FixtureGroup::Fields,
            seed: 1005,
            description: "Thermal gradient distribution under overload conditions",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 5,
                source_label: Some("Overload heat"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("field.diagnostic-confidence"),
            group: FixtureGroup::Fields,
            seed: 1006,
            description: "Probabilistic confidence metric mapping",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 6,
                source_label: Some("Diagnostic confidence"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("field.component-stress"),
            group: FixtureGroup::Fields,
            seed: 1007,
            description: "Mechanical or electrical stress concentration map",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 7,
                source_label: Some("Component stress"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("field.selection-density"),
            group: FixtureGroup::Fields,
            seed: 1008,
            description: "Continuous selection intensity field",
            provenance: FixtureProvenance {
                board: 'G',
                cell_index: 8,
                source_label: Some("Selection density"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        // ── Group H: Depth (8 fixtures) ───────────────────────────────────
        FixtureDef {
            id: FixtureId::from_static("depth.contact-shadow"),
            group: FixtureGroup::Depth,
            seed: 2001,
            description: "Tight ambient occlusion contact shadow",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 1,
                source_label: Some("Contact shadow"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("depth.component-lift-shadow"),
            group: FixtureGroup::Depth,
            seed: 2002,
            description: "Elevated surface shadow with diffused falloff",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 2,
                source_label: Some("Component lift shadow"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("depth.recessed-socket-shadow"),
            group: FixtureGroup::Depth,
            seed: 2003,
            description: "Interior cavity occlusion shadow",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 3,
                source_label: Some("Recessed socket shadow"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("depth.inset-panel-inner-shadow"),
            group: FixtureGroup::Depth,
            seed: 2004,
            description: "Recessed panel edge with directional inner shadow",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 4,
                source_label: Some("Inset panel border with inner shadow"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("depth.raised-instrument-subtle-bevel"),
            group: FixtureGroup::Depth,
            seed: 2005,
            description: "Subtle dual-tone linear edge bevel highlight",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 5,
                source_label: Some("Raised instrument body subtle bevel"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("depth.soft-edge-top-highlight"),
            group: FixtureGroup::Depth,
            seed: 2006,
            description: "Soft ambient top edge specular rim highlight",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 6,
                source_label: Some("Soft edge highlight top line"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("depth.rim-highlight-outline"),
            group: FixtureGroup::Depth,
            seed: 2007,
            description: "Continuous perimeter rim specular highlight",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 7,
                source_label: Some("Rim highlight outline"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("depth.shallow-bevel-3d-edge"),
            group: FixtureGroup::Depth,
            seed: 2008,
            description: "Steep prismatic edge bevel profile",
            provenance: FixtureProvenance {
                board: 'H',
                cell_index: 8,
                source_label: Some("Shallow bevel 3D edge"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        // ── Group I: Optical (8 fixtures) ─────────────────────────────────
        FixtureDef {
            id: FixtureId::from_static("optical.radial-fade"),
            group: FixtureGroup::Optical,
            seed: 3001,
            description: "Circular radial opacity falloff",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 1,
                source_label: Some("Radial fade"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("optical.directional-fade"),
            group: FixtureGroup::Optical,
            seed: 3002,
            description: "Linear directional opacity gradient",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 2,
                source_label: Some("Directional fade"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("optical.soft-rectangle-rounded"),
            group: FixtureGroup::Optical,
            seed: 3003,
            description: "Soft rounded box glow boundary",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 3,
                source_label: Some("Soft rectangle rounded"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("optical.soft-circle-radial"),
            group: FixtureGroup::Optical,
            seed: 3004,
            description: "Soft radial disc glow boundary",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 4,
                source_label: Some("Soft circle radial"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("optical.edge-vignette"),
            group: FixtureGroup::Optical,
            seed: 3005,
            description: "Corner darkening peripheral vignette",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 5,
                source_label: Some("Edge vignette corners darker"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("optical.local-focus"),
            group: FixtureGroup::Optical,
            seed: 3006,
            description: "Central luminance boost with ambient attenuation",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 6,
                source_label: Some("Local focus center bright"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("optical.energized-wire-glow"),
            group: FixtureGroup::Optical,
            seed: 3007,
            description: "Filament linear core with yellow halo bloom",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 7,
                source_label: Some("Energized wire yellow glow"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("optical.heat-region-glow"),
            group: FixtureGroup::Optical,
            seed: 3008,
            description: "Warm orange thermal emission glow",
            provenance: FixtureProvenance {
                board: 'I',
                cell_index: 8,
                source_label: Some("Heat region orange glow"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        // ── Group J: Materials (8 fixtures) ───────────────────────────────
        FixtureDef {
            id: FixtureId::from_static("material.painted-metal.matte-anthracite"),
            group: FixtureGroup::Materials,
            seed: 4001,
            description: "Deep neutral matte industrial coating",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 1,
                source_label: Some("Matte anthracite"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("material.painted-metal.signal-orange"),
            group: FixtureGroup::Materials,
            seed: 4002,
            description: "High-visibility warning signal orange coating",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 2,
                source_label: Some("Signal orange painted metal"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("material.painted-metal.beret-green"),
            group: FixtureGroup::Materials,
            seed: 4003,
            description: "Tactical muted military green coating",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 3,
                source_label: Some("Beret green painted metal"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("material.painted-metal.royal-blue"),
            group: FixtureGroup::Materials,
            seed: 4004,
            description: "Precision instrument royal blue coating",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 4,
                source_label: Some("Royal blue painted metal"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("material.brushed-aluminum"),
            group: FixtureGroup::Materials,
            seed: 4005,
            description: "Anisotropic linear brushed metallic surface",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 5,
                source_label: Some("Brushed aluminum"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("material.dark-anodized-aluminum"),
            group: FixtureGroup::Materials,
            seed: 4006,
            description: "Dark oxidized satin metallic substrate",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 6,
                source_label: Some("Dark anodized aluminum"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("material.warm-paper"),
            group: FixtureGroup::Materials,
            seed: 4007,
            description: "Organic fibrous diffuse paper substrate",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 7,
                source_label: Some("Warm paper"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("material.smoked-plastic"),
            group: FixtureGroup::Materials,
            seed: 4008,
            description: "Translucent smoked polycarbonate polymer",
            provenance: FixtureProvenance {
                board: 'J',
                cell_index: 8,
                source_label: Some("Smoked plastic"),
                source_artifact: GHIJ_SOURCE_ARTIFACT,
                source_sha256: GHIJ_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        // ── Group K: Microstructure (14 swatches) ─────────────────────────
        FixtureDef {
            id: FixtureId::from_static("micro.k01"),
            group: FixtureGroup::Microstructure,
            seed: 5001,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 1,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k02"),
            group: FixtureGroup::Microstructure,
            seed: 5002,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 2,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k03"),
            group: FixtureGroup::Microstructure,
            seed: 5003,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 3,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k04"),
            group: FixtureGroup::Microstructure,
            seed: 5004,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 4,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k05"),
            group: FixtureGroup::Microstructure,
            seed: 5005,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 5,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k06"),
            group: FixtureGroup::Microstructure,
            seed: 5006,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 6,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k07"),
            group: FixtureGroup::Microstructure,
            seed: 5007,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 7,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k08"),
            group: FixtureGroup::Microstructure,
            seed: 5008,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 8,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k09"),
            group: FixtureGroup::Microstructure,
            seed: 5009,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 9,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k10"),
            group: FixtureGroup::Microstructure,
            seed: 5010,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 10,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k11"),
            group: FixtureGroup::Microstructure,
            seed: 5011,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 11,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k12"),
            group: FixtureGroup::Microstructure,
            seed: 5012,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 12,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k13"),
            group: FixtureGroup::Microstructure,
            seed: 5013,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 13,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
        FixtureDef {
            id: FixtureId::from_static("micro.k14"),
            group: FixtureGroup::Microstructure,
            seed: 5014,
            description: "",
            provenance: FixtureProvenance {
                board: 'K',
                cell_index: 14,
                source_label: None,
                source_artifact: K_SOURCE_ARTIFACT,
                source_sha256: K_SOURCE_SHA256,
            },
            capability: CapabilityClass::Unknown,
            determinism: FixtureDeterminism::Deterministic,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_fixture_catalog_completeness() {
        let all = FixtureCatalog::all();
        assert_eq!(all.len(), 46, "Catalog must contain exactly 46 fixtures");
        assert_eq!(FixtureCatalog::count(), 46);

        let mut ids = HashSet::new();
        let mut seeds = HashSet::new();

        for f in all {
            assert!(
                ids.insert(f.id.as_str()),
                "Duplicate fixture id: {}",
                f.id.as_str()
            );
            assert!(seeds.insert(f.seed), "Duplicate seed: {}", f.seed);
            assert_eq!(
                f.capability,
                CapabilityClass::Unknown,
                "All fixtures in RENDER-00 must be UNKNOWN"
            );
            assert_eq!(
                f.determinism,
                FixtureDeterminism::Deterministic,
                "All fixtures in RENDER-00 must be Deterministic"
            );

            // Validate provenance
            match f.group {
                FixtureGroup::Fields
                | FixtureGroup::Depth
                | FixtureGroup::Optical
                | FixtureGroup::Materials => {
                    assert_eq!(f.provenance.source_artifact, GHIJ_SOURCE_ARTIFACT);
                    assert_eq!(f.provenance.source_sha256, GHIJ_SOURCE_SHA256);
                    assert!(f.provenance.source_label.is_some());
                    assert!(f.provenance.cell_index >= 1 && f.provenance.cell_index <= 8);
                }
                FixtureGroup::Microstructure => {
                    assert_eq!(f.provenance.source_artifact, K_SOURCE_ARTIFACT);
                    assert_eq!(f.provenance.source_sha256, K_SOURCE_SHA256);
                    assert_eq!(f.provenance.source_label, None);
                    assert!(f.provenance.cell_index >= 1 && f.provenance.cell_index <= 14);
                }
            }
        }

        assert_eq!(FixtureCatalog::for_group(FixtureGroup::Fields).count(), 8);
        assert_eq!(FixtureCatalog::for_group(FixtureGroup::Depth).count(), 8);
        assert_eq!(FixtureCatalog::for_group(FixtureGroup::Optical).count(), 8);
        assert_eq!(
            FixtureCatalog::for_group(FixtureGroup::Materials).count(),
            8
        );
        assert_eq!(
            FixtureCatalog::for_group(FixtureGroup::Microstructure).count(),
            14
        );
    }

    #[test]
    fn test_fixture_catalog_find() {
        let id = FixtureId::from_static("field.signed-voltage");
        let found = FixtureCatalog::find(&id);
        assert!(found.is_some());
        let f = found.unwrap();
        assert_eq!(f.seed, 1002);
        assert_eq!(f.group, FixtureGroup::Fields);
        assert_eq!(f.provenance.source_label, Some("Signed voltage"));

        let found_str = FixtureCatalog::find_by_str("material.painted-metal.signal-orange");
        assert!(found_str.is_some());
        assert_eq!(found_str.unwrap().seed, 4002);

        let not_found = FixtureCatalog::find_by_str("invalid.id");
        assert!(not_found.is_none());
    }
}
