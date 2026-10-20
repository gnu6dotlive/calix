use crate::render_lab::capability::CapabilityClass;
use crate::render_lab::recipe::RecipeCatalog;
use serde::Serialize;

/// Backend under confrontation in RENDER-01.
pub const STOCK_BACKEND_ID: &str = "gpui-stock-0.2.2";

/// Evidence artifacts live under this docs prefix (ROI captures plus
/// per-fixture confrontation manifests).
pub const EVIDENCE_PREFIX: &str = "evidence/render01/";

/// One backend-specific capability observation.
///
/// The catalog owns corpus identity; the ledger owns earned verdicts.
/// `manifest_hash` is the SHA-256 of the normalized preview-pixel buffer
/// measured during the evidence run, never a container-file hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CapabilityObservation {
    pub fixture: &'static str,
    pub backend: &'static str,
    pub verdict: CapabilityClass,
    pub recipe: &'static str,
    pub evidence: &'static str,
    pub manifest_hash: &'static str,
    pub caveats: &'static str,
}

/// Ratified observation table. Extended wave by wave as evidence lands;
/// RENDER-01 closes at 46/46.
pub struct CapabilityLedger;

static OBSERVATIONS: &[CapabilityObservation] = &[
    CapabilityObservation {
        fixture: "field.zero-positive-scalar",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "g01-linear-scalar-gradient/r1",
        evidence: "evidence/render01/g/field.zero-positive-scalar.roi.png",
        manifest_hash: "eb5c1c8d5c811fb784a062f58a38349476eae27b9907ed620c1e6f072175a60d",
        caveats: "Direct 1D linear scalar field natively supported by GPUI linear_gradient. 2D non-linear gradients require composition.",
    },
    CapabilityObservation {
        fixture: "field.signed-voltage",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "g02-bipolar-split/r1",
        evidence: "evidence/render01/g/field.signed-voltage.roi.png",
        manifest_hash: "2eaa3d665f278484ff58e7a8fe2303091ef6f91e91dea8418a35a675dbb60e2f",
        caveats: "Zero-crossing threshold composed from two adjacent opposite-polarity gradients. Sharp boundary or nonlinear transition requires quad tessellation or texture.",
    },
    CapabilityObservation {
        fixture: "field.current-magnitude",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "g03-magnitude-contour-stops/r1",
        evidence: "evidence/render01/g/field.current-magnitude.roi.png",
        manifest_hash: "70e69bacf9e0c1b08d0a566f6accbde0cb136fa2f68207da93c9ad2b09c9f864",
        caveats: "Magnitude ramp composable via multi-stop linear gradient; discrete isoline steps achieved by stepped color stops.",
    },
    CapabilityObservation {
        fixture: "field.current-direction",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "g04-streamline-vector-field/r1",
        evidence: "evidence/render01/g/field.current-direction.roi.png",
        manifest_hash: "6486a1760ea6eea884ed7d8e26b06f1683a13c8b75f1634a64e4cf49ed27643f",
        caveats: "Directional vector field expressed via composed vector paths; continuous dense vector direction field requires texture.",
    },
    CapabilityObservation {
        fixture: "field.overload-heat",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "g05-thermal-multistop-gradient/r1",
        evidence: "evidence/render01/g/field.overload-heat.roi.png",
        manifest_hash: "c6d17989dc042060946ef48849d4ebf82315b9b1be8fe7222d6066ede389cf96",
        caveats: "Thermal palette composed from layered linear gradients and centered radiant oval.",
    },
    CapabilityObservation {
        fixture: "field.diagnostic-confidence",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "g06-probabilistic-confidence-band/r1",
        evidence: "evidence/render01/g/field.diagnostic-confidence.roi.png",
        manifest_hash: "ec2e2374c4aa10115403d416acc9af5931324fe2ea7ad5ee3e2d8de78c6ad486",
        caveats: "1D probability density cleanly maps to single stock linear_gradient.",
    },
    CapabilityObservation {
        fixture: "field.component-stress",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "g07-stress-concentration-contour/r1",
        evidence: "evidence/render01/g/field.component-stress.roi.png",
        manifest_hash: "eb9c1370938ec54e61393bf330af2b4d0d26d878059a1afb99fd22879ba10df0",
        caveats: "Stress concentration composed from concentric stress boundary shells.",
    },
    CapabilityObservation {
        fixture: "field.selection-density",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "g08-continuous-density-plane/r1",
        evidence: "evidence/render01/g/field.selection-density.roi.png",
        manifest_hash: "e75bdb57be488f710297ee2ccf9b5d8151329e1f61a5695752fed2b074dbfd7a",
        caveats: "2D bilinear density plane approximated by cross-fading orthogonal linear gradients.",
    },
    CapabilityObservation {
        fixture: "depth.contact-shadow",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "h01-single-box-shadow/r1",
        evidence: "evidence/render01/h/depth.contact-shadow.roi.png",
        manifest_hash: "5069768d7b6c85a4d222f63d1782804595a988908b8c763b218ca27545c120da",
        caveats: "Tight contact occlusion cleanly produced by stock BoxShadow with low blur radius.",
    },
    CapabilityObservation {
        fixture: "depth.component-lift-shadow",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "h02-diffused-elevation-shadow/r1",
        evidence: "evidence/render01/h/depth.component-lift-shadow.roi.png",
        manifest_hash: "d80e3f348b13eb93efedae3962154aa42c45df8ae36d82627323dbb8e5bf77c3",
        caveats: "Elevation penumbra produced by stock BoxShadow with larger blur and vertical offset.",
    },
    CapabilityObservation {
        fixture: "depth.recessed-socket-shadow",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "h03-cavity-occlusion-bevel/r1",
        evidence: "evidence/render01/h/depth.recessed-socket-shadow.roi.png",
        manifest_hash: "3babe69a08f44117b15b48243b6cbc8c6baf1493d99b289872eac01a21bf9660",
        caveats: "GPUI has no native inner shadow; cavity depth is composed from directional 1px/2px inner bevel borders and linear cavity gradient.",
    },
    CapabilityObservation {
        fixture: "depth.inset-panel-inner-shadow",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "h04-inset-panel-directional-bevel/r1",
        evidence: "evidence/render01/h/depth.inset-panel-inner-shadow.roi.png",
        manifest_hash: "29a5038fa2c20c484c61cc7b12081eb477e9a83ffb366fcdaf95c010c72ba376",
        caveats: "Inner shadow illusion composed from directional border strokes and linear falloff child.",
    },
    CapabilityObservation {
        fixture: "depth.raised-instrument-subtle-bevel",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "h05-dual-tone-linear-bevel/r1",
        evidence: "evidence/render01/h/depth.raised-instrument-subtle-bevel.roi.png",
        manifest_hash: "51cca0660a936cf280f2819d24e811d48158923e040b52708a0c5da89c69f140",
        caveats: "Physical bevel profile composed from opposing edge highlights, outer shadow, and subtle body gradient.",
    },
    CapabilityObservation {
        fixture: "depth.soft-edge-top-highlight",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "h06-ambient-top-highlight-rim/r1",
        evidence: "evidence/render01/h/depth.soft-edge-top-highlight.roi.png",
        manifest_hash: "7292d844507490bebd307fb057eed7ecf8e6e33039604febc82379ce7b69bb9a",
        caveats: "Single top-edge specular line directly expressible via stock border_t_1 and border_color.",
    },
    CapabilityObservation {
        fixture: "depth.rim-highlight-outline",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "h07-continuous-perimeter-rim/r1",
        evidence: "evidence/render01/h/depth.rim-highlight-outline.roi.png",
        manifest_hash: "9402939a065c3c4493009a2b5de03af12315035ae2b1a49e05523b167e954363",
        caveats: "Uniform perimeter highlight directly expressible via stock border_1 and border_color.",
    },
    CapabilityObservation {
        fixture: "depth.shallow-bevel-3d-edge",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "h08-prismatic-shallow-bevel/r1",
        evidence: "evidence/render01/h/depth.shallow-bevel-3d-edge.roi.png",
        manifest_hash: "0dd8d52d7da31119ace080b74d78cc69f0038156a6ef053cfa224b682b0ac0cf",
        caveats: "Multi-stage prismatic bevel composed of concentric nested borders and contrasting elevation shadows.",
    },
    CapabilityObservation {
        fixture: "optical.radial-fade",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "i01-concentric-falloff/r1",
        evidence: "evidence/render01/i/optical.radial-fade.roi.png",
        manifest_hash: "ebeadcf55c3c6e5e32098a13dc35df84d7c75b30c3b7784b262e2fc2f00c10db",
        caveats: "Stock GPUI lacks radial gradient primitive; radial fade is approximated by concentric geometric discs. High ring count has tessellation cost.",
    },
    CapabilityObservation {
        fixture: "optical.directional-fade",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "i02-linear-directional-fade/r1",
        evidence: "evidence/render01/i/optical.directional-fade.roi.png",
        manifest_hash: "42349ccdd5185b04bb3f90393aa2dafd98233fd74d44e8f0c708500c092eeebf",
        caveats: "Linear optical fade directly supported by stock linear_gradient with alpha color stops.",
    },
    CapabilityObservation {
        fixture: "optical.soft-rectangle-rounded",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "i03-soft-rounded-box-glow/r1",
        evidence: "evidence/render01/i/optical.soft-rectangle-rounded.roi.png",
        manifest_hash: "d2c45d34e639ec3c71ccae25ecf6a311ce3069c444b9255725db1be098f71278",
        caveats: "Soft rounded glow boundary cleanly produced by stock BoxShadow with zero offset and positive spread.",
    },
    CapabilityObservation {
        fixture: "optical.soft-circle-radial",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "i04-radial-disc-glow-boundary/r1",
        evidence: "evidence/render01/i/optical.soft-circle-radial.roi.png",
        manifest_hash: "c1c8d723c5e7b9f17e0d186cd8b1952782afbd2718ba067624005c2aa41e29da",
        caveats: "Two-scale glow halo composed from nested circular disc and multi-lobe BoxShadow.",
    },
    CapabilityObservation {
        fixture: "optical.edge-vignette",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "i05-corner-darkening-vignette/r1",
        evidence: "evidence/render01/i/optical.edge-vignette.roi.png",
        manifest_hash: "0d3c444258e594ad05bc9cd8c7bfdcfe1b195a33edc9b451571c580c3e8efc8a",
        caveats: "True radial vignette composed of peripheral linear gradients and corner shadow anchors.",
    },
    CapabilityObservation {
        fixture: "optical.local-focus",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "i06-central-luminance-boost/r1",
        evidence: "evidence/render01/i/optical.local-focus.roi.png",
        manifest_hash: "6d0933186418194c544c8a0efcfc846a47ba11d2a5f0255e7cacc65ecfbcf9ec",
        caveats: "Focus spotlight composed of centered illuminated disc and ambient darkening mask.",
    },
    CapabilityObservation {
        fixture: "optical.energized-wire-glow",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "i07-filament-core-bloom/r1",
        evidence: "evidence/render01/i/optical.energized-wire-glow.roi.png",
        manifest_hash: "d13ac542ef4aa6871edd81c6d5bf60aa6ec0c37033ff68ef681d409949c798a4",
        caveats: "Bloom illusion composed of crisp 2px central filament and dual-stage soft BoxShadow glow.",
    },
    CapabilityObservation {
        fixture: "optical.heat-region-glow",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "i08-thermal-emission-envelope/r1",
        evidence: "evidence/render01/i/optical.heat-region-glow.roi.png",
        manifest_hash: "e54427cc0964e639558adc8c02f4aa15b150f36488b94527deaa6a5930896764",
        caveats: "Volumetric heat glow approximated by layered concentric thermal discs with nonlinear opacity falloff.",
    },
    CapabilityObservation {
        fixture: "material.painted-metal.matte-anthracite",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "j01-matte-anthracite-coating/r1",
        evidence: "evidence/render01/j/material.painted-metal.matte-anthracite.roi.png",
        manifest_hash: "bd979004d0601cf4345c4b82bb31a726549934e8acd63d59ddeac617bb336ff6",
        caveats: "Diffuse matte industrial finish cleanly represented by stock solid fill with subtle structural edge highlight.",
    },
    CapabilityObservation {
        fixture: "material.painted-metal.signal-orange",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "j02-signal-orange-coating/r1",
        evidence: "evidence/render01/j/material.painted-metal.signal-orange.roi.png",
        manifest_hash: "ffe09cdce248743aa0043ca5fc7755ddc53a0b0de4ece7aabbe7d05ab288ee5a",
        caveats: "Vibrant painted metal finish cleanly represented by stock solid fill, 1px highlight border and subtle top sheen.",
    },
    CapabilityObservation {
        fixture: "material.painted-metal.beret-green",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "j03-beret-green-coating/r1",
        evidence: "evidence/render01/j/material.painted-metal.beret-green.roi.png",
        manifest_hash: "d7b9145bfd6b0b26fe973d44bbce050d1f04fe802923269a6ed3c61507e2c4d2",
        caveats: "Muted tactical paint cleanly represented by stock solid fill, subtle gradient, and edge border.",
    },
    CapabilityObservation {
        fixture: "material.painted-metal.royal-blue",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Native,
        recipe: "j04-royal-blue-coating/r1",
        evidence: "evidence/render01/j/material.painted-metal.royal-blue.roi.png",
        manifest_hash: "7720604f14100922ed59f2c96d0d681c3c7f148cf67ae775ce18bb051873b289",
        caveats: "Precision instrument enamel cleanly represented by stock solid fill, subtle gradient, and edge border.",
    },
    CapabilityObservation {
        fixture: "material.brushed-aluminum",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "j05-brushed-sphere-texture/r1",
        evidence: "evidence/render01/j/material.brushed-aluminum.roi.png",
        manifest_hash: "ed697fda33086db5214f2d5e118b25efffa2992ca44aeb93c8856a32502e850f",
        caveats: "Directional anisotropic grain and micro-streaks cannot be synthesized via vector primitives; proven through deterministic immutable memory texture.",
    },
    CapabilityObservation {
        fixture: "material.dark-anodized-aluminum",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "j06-dark-anodized-satin/r1",
        evidence: "evidence/render01/j/material.dark-anodized-aluminum.roi.png",
        manifest_hash: "e83fbdc4a4f31cd1a135a083d9c51b920cd0ac949f6be24c2a9757c6f80d0843",
        caveats: "Satin anodized sheen composed of subtle multi-stage linear gradient, low-contrast specular rim, and dark metallic fill.",
    },
    CapabilityObservation {
        fixture: "material.warm-paper",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "j07-warm-paper-fibrous-texture/r1",
        evidence: "evidence/render01/j/material.warm-paper.roi.png",
        manifest_hash: "311da5160cf2298a6cad3621ee1d579bb3cc0c019f2d1d7588e87169e72c4472",
        caveats: "Paper fiber microstructure and stochastic pulp variation require texture synthesis; verified via immutable memory texture.",
    },
    CapabilityObservation {
        fixture: "material.smoked-plastic",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::Composable,
        recipe: "j08-smoked-polycarbonate-translucent/r1",
        evidence: "evidence/render01/j/material.smoked-plastic.roi.png",
        manifest_hash: "204a4e9a78d8d3d5952b97b2a84d511a7386cd3e308fce3353bfd9c45de2b3a9",
        caveats: "Optical transmission and surface reflection composed of translucent tinted fill, inner highlight line, and backdrop contrast.",
    },
    CapabilityObservation {
        fixture: "micro.k01",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k01-uniform-noise-texture/r1",
        evidence: "evidence/render01/k/micro.k01.roi.png",
        manifest_hash: "715b027a95fb44907c75119ad4bdfd64ef6ff192fb07701a50bcbd294cc00585",
        caveats: "Uniform high-frequency stochastic noise synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k02",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k02-stratified-jitter-texture/r1",
        evidence: "evidence/render01/k/micro.k02.roi.png",
        manifest_hash: "afd610f235e3e9ffba8a9cfb79bbc282cb925fa0bba920c18a6e750b7fbbf460",
        caveats: "Stratified jitter grid marks synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k03",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k03-fine-grit-texture/r1",
        evidence: "evidence/render01/k/micro.k03.roi.png",
        manifest_hash: "a76c0102ec6099bcdc9de4e95bc32dfd9c6eda0905960cac27b175ef9b0fdff5",
        caveats: "Fine multi-scale grit with stochastic particle points synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k04",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k04-brushed-micro-texture/r1",
        evidence: "evidence/render01/k/micro.k04.roi.png",
        manifest_hash: "44cdb776c604707c688f893a128b6b9ea8b32dedee3439ae04a9865de1149dec",
        caveats: "Dense directional micro-scratch field synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k05",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k05-anisotropic-grain-texture/r1",
        evidence: "evidence/render01/k/micro.k05.roi.png",
        manifest_hash: "e87a46d19c1078ff5c22bb018effee6a0c4096e580085580688d14262bd280a8",
        caveats: "Anisotropic directional noise field synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k06",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k06-cellular-pattern-texture/r1",
        evidence: "evidence/render01/k/micro.k06.roi.png",
        manifest_hash: "fb68081c7579409e08cc2ea8431ecbd3c2c2a9a285d04fdaa78531d7e749e0e6",
        caveats: "Cellular Voronoi distance field synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k07",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k07-stochastic-stipple-texture/r1",
        evidence: "evidence/render01/k/micro.k07.roi.png",
        manifest_hash: "f282622f8a865f7033c083c0cc62f5f9bafed05e1058f48faa0bec45199158a9",
        caveats: "Stochastic stipple point density synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k08",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k08-value-noise-lattice-texture/r1",
        evidence: "evidence/render01/k/micro.k08.roi.png",
        manifest_hash: "c2d2a9576b1ebbd3c20878b2220d4e3182bc3c7cd906619537b1e8420e6e7fb9",
        caveats: "Bilinear value noise lattice synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k09",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k09-halftone-mesh-texture/r1",
        evidence: "evidence/render01/k/micro.k09.roi.png",
        manifest_hash: "01925e672d4c5aa1a59fb71b19da87af2fd052713d3714e883034c63317017a4",
        caveats: "Geometric halftone dot mesh synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k10",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k10-woven-matrix-texture/r1",
        evidence: "evidence/render01/k/micro.k10.roi.png",
        manifest_hash: "352b67275609b3f05a516f389c9df85cbd9a56ba331f4195185e2a7ce4e6d3ad",
        caveats: "Interleaved woven matrix threads synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k11",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k11-crater-relief-texture/r1",
        evidence: "evidence/render01/k/micro.k11.roi.png",
        manifest_hash: "996b5b17156eba665e99c0647aaa93135103613432f338ca885bcbe773af2df8",
        caveats: "Crater pore depressions with directional lighting synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k12",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k12-etched-fiber-texture/r1",
        evidence: "evidence/render01/k/micro.k12.roi.png",
        manifest_hash: "5faae4c199e8e651213a4704a1bfbc9f6eeaba954fc25810cce675836d6bf070",
        caveats: "Curved etched fiber segments synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k13",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k13-fine-dither-texture/r1",
        evidence: "evidence/render01/k/micro.k13.roi.png",
        manifest_hash: "8e92e584395672242e669038cba13d0c53a70ffa0ea9ea092546280ef7f4389b",
        caveats: "Ordered Bayer matrix dither synthesized into deterministic immutable texture buffer.",
    },
    CapabilityObservation {
        fixture: "micro.k14",
        backend: STOCK_BACKEND_ID,
        verdict: CapabilityClass::TextureProof,
        recipe: "k14-coarse-grain-texture/r1",
        evidence: "evidence/render01/k/micro.k14.roi.png",
        manifest_hash: "f448eba7e86ee9fa1e4caa4f718f07a7250f4bb4dc0ba0cdc9783de2983fb18b",
        caveats: "Coarse block grain clusters synthesized into deterministic immutable texture buffer.",
    },
];

impl CapabilityLedger {
    pub fn all() -> &'static [CapabilityObservation] {
        OBSERVATIONS
    }

    pub fn count() -> usize {
        OBSERVATIONS.len()
    }

    pub fn find(fixture_id: &str) -> Option<&'static CapabilityObservation> {
        OBSERVATIONS.iter().find(|o| o.fixture == fixture_id)
    }

    /// Structural validation over one observation, shared by the table
    /// validator and unit tests.
    fn validate_one(
        obs: &CapabilityObservation,
        seen: &mut std::collections::HashSet<&'static str>,
    ) -> Result<(), String> {
        if !seen.insert(obs.fixture) {
            return Err(format!("Duplicate ledger entry for '{}'", obs.fixture));
        }
        if obs.backend != STOCK_BACKEND_ID {
            return Err(format!(
                "Observation '{}' targets backend '{}', expected '{STOCK_BACKEND_ID}'",
                obs.fixture, obs.backend
            ));
        }
        if obs.verdict == CapabilityClass::ShaderRequired {
            return Err(format!(
                "Observation '{}' claims ShaderRequired, forbidden before RENDER-03",
                obs.fixture
            ));
        }
        match RecipeCatalog::find(obs.fixture) {
            Some(recipe) if recipe.id == obs.recipe => {}
            _ => {
                return Err(format!(
                    "Observation '{}' references unknown recipe '{}'",
                    obs.fixture, obs.recipe
                ))
            }
        }
        if !obs.evidence.starts_with(EVIDENCE_PREFIX) {
            return Err(format!(
                "Observation '{}' evidence '{}' outside '{EVIDENCE_PREFIX}'",
                obs.fixture, obs.evidence
            ));
        }
        if obs.manifest_hash.len() != 64
            || !obs.manifest_hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(format!(
                "Observation '{}' has malformed manifest hash",
                obs.fixture
            ));
        }
        if obs.caveats.is_empty() {
            return Err(format!("Observation '{}' lacks caveats", obs.fixture));
        }
        Ok(())
    }

    /// Structural validation: no duplicates, verdicts within the RENDER-01
    /// allowed set, recipe cross-checked against the recipe catalog, hash
    /// format, evidence prefix, mandatory caveats.
    pub fn validate() -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        for obs in OBSERVATIONS {
            Self::validate_one(obs, &mut seen)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    fn sample(verdict: CapabilityClass) -> CapabilityObservation {
        CapabilityObservation {
            fixture: "field.signed-voltage",
            backend: STOCK_BACKEND_ID,
            verdict,
            recipe: "g02-bipolar-split/r1",
            evidence: "evidence/render01/g/field.signed-voltage.roi.png",
            manifest_hash: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            caveats: "Sample observation for validation tests.",
        }
    }

    fn validate_sample(obs: &CapabilityObservation) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();
        CapabilityLedger::validate_one(obs, &mut seen)
    }

    #[test]
    fn test_ledger_constants_and_committed_table() {
        assert_eq!(STOCK_BACKEND_ID, "gpui-stock-0.2.2");
        assert_eq!(EVIDENCE_PREFIX, "evidence/render01/");
        CapabilityLedger::validate().expect("Committed ledger must validate");
    }

    #[test]
    fn test_observation_serializes_with_verdict() {
        let json = serde_json::to_string(&sample(CapabilityClass::Composable)).unwrap();
        assert!(json.contains("\"COMPOSABLE\""));
        assert!(json.contains("field.signed-voltage"));
    }

    #[test]
    fn test_validate_accepts_allowed_verdicts() {
        for verdict in [
            CapabilityClass::Unknown,
            CapabilityClass::Native,
            CapabilityClass::Composable,
            CapabilityClass::TextureProof,
        ] {
            validate_sample(&sample(verdict)).expect("allowed verdict must validate");
        }
    }

    #[test]
    fn test_validate_rejects_shader_required() {
        let err = validate_sample(&sample(CapabilityClass::ShaderRequired)).unwrap_err();
        assert!(err.contains("ShaderRequired"));
    }

    #[test]
    fn test_validate_rejects_malformed_entries() {
        let mut bad_hash = sample(CapabilityClass::Native);
        bad_hash.manifest_hash = "not-a-hash";
        assert!(validate_sample(&bad_hash).is_err());

        let mut bad_recipe = sample(CapabilityClass::Native);
        bad_recipe.recipe = "g02-invented/r9";
        assert!(validate_sample(&bad_recipe).is_err());

        let mut bad_evidence = sample(CapabilityClass::Native);
        bad_evidence.evidence = "/tmp/stray.png";
        assert!(validate_sample(&bad_evidence).is_err());

        let mut no_caveats = sample(CapabilityClass::Native);
        no_caveats.caveats = "";
        assert!(validate_sample(&no_caveats).is_err());
    }
}
