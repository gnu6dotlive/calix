use crate::render_lab::catalog::FixtureCatalog;
use crate::render_lab::diagnostics::DiagnosticEntry;
use crate::render_lab::fixture::FixtureId;
use crate::visual_style::VisualStyleId;
use gpui::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "t", rename_all = "kebab-case")]
pub enum ClockMode {
    Realtime,
    Frozen(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TopologyVariant {
    #[default]
    FloatingIsland,
    FullBand,
    PerimeterHug,
}

impl TopologyVariant {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "floating-island" | "floating_island" | "island" => Ok(Self::FloatingIsland),
            "full-band" | "full_band" | "band" => Ok(Self::FullBand),
            "perimeter-hug" | "perimeter_hug" | "perimeter" => Ok(Self::PerimeterHug),
            other => Err(format!(
                "Unknown topology variant '{other}'. Expected 'floating-island', 'full-band', or 'perimeter-hug'"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::FloatingIsland => "floating-island",
            Self::FullBand => "full-band",
            Self::PerimeterHug => "perimeter-hug",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MotionVariant {
    #[default]
    Classic,
    Smooth,
    Elastic,
    Liquid,
    ReducedMotion,
}

impl MotionVariant {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "classic" => Ok(Self::Classic),
            "smooth" => Ok(Self::Smooth),
            "elastic" => Ok(Self::Elastic),
            "liquid" => Ok(Self::Liquid),
            "reduced-motion" | "reduced_motion" | "reduced" => Ok(Self::ReducedMotion),
            other => Err(format!(
                "Unknown motion variant '{other}'. Expected 'classic', 'smooth', 'elastic', 'liquid', or 'reduced-motion'"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Smooth => "smooth",
            Self::Elastic => "elastic",
            Self::Liquid => "liquid",
            Self::ReducedMotion => "reduced-motion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QualityLevel {
    #[default]
    Stock,
}

impl QualityLevel {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "stock" => Ok(Self::Stock),
            other => Err(format!(
                "Unknown quality level '{other}'. Only 'stock' is supported in RENDER-00"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stock => "stock",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderLabEvent {
    FixtureChanged(Option<FixtureId>),
    TopologyChanged(TopologyVariant),
    MotionChanged(MotionVariant),
    QualityChanged(QualityLevel),
    ClockChanged(ClockMode),
    StyleChanged(VisualStyleId),
}

pub struct RenderLabState {
    pub active_fixture: Option<FixtureId>,
    pub active_topology: TopologyVariant,
    pub active_motion: MotionVariant,
    pub active_quality: QualityLevel,
    pub active_style: VisualStyleId,
    pub clock: ClockMode,
    pub diagnostics: Vec<DiagnosticEntry>,
}

impl EventEmitter<RenderLabEvent> for RenderLabState {}

impl Default for RenderLabState {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderLabState {
    pub fn new() -> Self {
        Self {
            active_fixture: None,
            active_topology: TopologyVariant::default(),
            active_motion: MotionVariant::default(),
            active_quality: QualityLevel::default(),
            active_style: VisualStyleId::Standard,
            clock: ClockMode::Realtime,
            diagnostics: vec![DiagnosticEntry::info("Render Lab initialized (RENDER-00)")],
        }
    }

    pub fn set_fixture(&mut self, id: Option<FixtureId>, cx: &mut Context<Self>) {
        if self.active_fixture != id {
            self.active_fixture = id.clone();
            cx.emit(RenderLabEvent::FixtureChanged(id));
            cx.notify();
        }
    }

    pub fn set_fixture_by_id_str(
        &mut self,
        id_str: &str,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let trimmed = id_str.trim();
        if trimmed.is_empty()
            || trimmed.eq_ignore_ascii_case("none")
            || trimmed.eq_ignore_ascii_case("clear")
        {
            self.set_fixture(None, cx);
            return Ok(());
        }
        let fixture_id = FixtureId::new(trimmed).map_err(|e| e.to_string())?;
        if FixtureCatalog::find(&fixture_id).is_none() {
            return Err(format!("Fixture '{id_str}' not found in catalog"));
        }
        self.set_fixture(Some(fixture_id), cx);
        Ok(())
    }

    pub fn set_topology(&mut self, topology: TopologyVariant, cx: &mut Context<Self>) {
        if self.active_topology != topology {
            self.active_topology = topology;
            cx.emit(RenderLabEvent::TopologyChanged(topology));
            cx.notify();
        }
    }

    pub fn set_motion(&mut self, motion: MotionVariant, cx: &mut Context<Self>) {
        if self.active_motion != motion {
            self.active_motion = motion;
            cx.emit(RenderLabEvent::MotionChanged(motion));
            cx.notify();
        }
    }

    pub fn set_quality(&mut self, quality: QualityLevel, cx: &mut Context<Self>) {
        if self.active_quality != quality {
            self.active_quality = quality;
            cx.emit(RenderLabEvent::QualityChanged(quality));
            cx.notify();
        }
    }

    pub fn set_clock(&mut self, clock: ClockMode, cx: &mut Context<Self>) {
        if self.clock != clock {
            self.clock = clock;
            cx.emit(RenderLabEvent::ClockChanged(clock));
            cx.notify();
        }
    }

    pub fn set_style(&mut self, style: VisualStyleId, cx: &mut Context<Self>) {
        if self.active_style != style {
            self.active_style = style;
            cx.emit(RenderLabEvent::StyleChanged(style));
            cx.notify();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_render_lab_state_defaults() {
        let state = RenderLabState::new();
        assert_eq!(state.active_fixture, None);
        assert_eq!(state.active_topology, TopologyVariant::FloatingIsland);
        assert_eq!(state.active_motion, MotionVariant::Classic);
        assert_eq!(state.active_quality, QualityLevel::Stock);
        assert_eq!(state.active_style, VisualStyleId::Standard);
        assert_eq!(state.clock, ClockMode::Realtime);
        assert!(!state.diagnostics.is_empty());
    }

    #[test]
    fn test_topology_parse() {
        assert_eq!(
            TopologyVariant::parse("floating-island").unwrap(),
            TopologyVariant::FloatingIsland
        );
        assert_eq!(
            TopologyVariant::parse("full-band").unwrap(),
            TopologyVariant::FullBand
        );
        assert_eq!(
            TopologyVariant::parse("perimeter-hug").unwrap(),
            TopologyVariant::PerimeterHug
        );
        assert!(TopologyVariant::parse("invalid").is_err());
    }

    #[test]
    fn test_motion_parse() {
        assert_eq!(
            MotionVariant::parse("classic").unwrap(),
            MotionVariant::Classic
        );
        assert_eq!(
            MotionVariant::parse("smooth").unwrap(),
            MotionVariant::Smooth
        );
        assert_eq!(
            MotionVariant::parse("elastic").unwrap(),
            MotionVariant::Elastic
        );
        assert_eq!(
            MotionVariant::parse("liquid").unwrap(),
            MotionVariant::Liquid
        );
        assert_eq!(
            MotionVariant::parse("reduced-motion").unwrap(),
            MotionVariant::ReducedMotion
        );
        assert!(MotionVariant::parse("invalid").is_err());
    }

    #[test]
    fn test_quality_parse() {
        assert_eq!(QualityLevel::parse("stock").unwrap(), QualityLevel::Stock);
        assert!(QualityLevel::parse("high").is_err());
    }

    #[test]
    fn test_clock_mode_serde() {
        let rt = ClockMode::Realtime;
        let rt_json = serde_json::to_string(&rt).unwrap();
        assert_eq!(rt_json, "{\"mode\":\"realtime\"}");
        let deserialized_rt: ClockMode = serde_json::from_str(&rt_json).unwrap();
        assert_eq!(deserialized_rt, rt);

        let frozen = ClockMode::Frozen(0.5);
        let frozen_json = serde_json::to_string(&frozen).unwrap();
        assert_eq!(frozen_json, "{\"mode\":\"frozen\",\"t\":0.5}");
        let deserialized_frozen: ClockMode = serde_json::from_str(&frozen_json).unwrap();
        assert_eq!(deserialized_frozen, frozen);
    }
}
