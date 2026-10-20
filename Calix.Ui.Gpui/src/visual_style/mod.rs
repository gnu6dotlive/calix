pub mod paint;
pub mod profile;
pub mod projection;
pub mod resolver;
pub mod roles;
pub mod standard;
pub mod transparency;

pub use paint::apply_surface_projection;
pub use profile::{ColorScheme, VisualStyleId, VisualStyleProfile, VisualStyleRegistry};
pub use projection::SurfacePaintProjection;
pub use resolver::{AppearanceStatus, EffectKind, EffectResolution, StyleProjection};
pub use roles::{SurfaceRole, SurfaceTreatment};
