use serde::{Deserialize, Serialize};

/// Control protocol version:
/// - v1: Initial Calix GPUI runtime control (navigation, search, view mode, inspection)
/// - v2: Render Lab baseline surface commands (open, fixture, material, topology, motion, quality, time, status)
/// - v3: RenderLabStyle command & schema v2 manifest fields (recipe, seed, diagnostics) for RENDER-01 confrontation
/// - v4: RenderLabMetrics, RenderLabRecipes, RenderLabRecipe for RENDER-02 structural telemetry
pub const CONTROL_PROTOCOL_VERSION: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlRequest {
    pub version: u32,
    pub command: ControlCommand,
}

impl ControlRequest {
    pub fn new(command: ControlCommand) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            command,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "payload", rename_all = "kebab-case")]
pub enum ControlCommand {
    Open,
    Focus,
    Quit,
    Status,
    Navigate { destination: String },
    /// Change the scope of the package surface, without leaving the place.
    Scope { scope: String },
    Search { query: String },
    View { mode: String },
    Inspect { package: String },
    Inspector { tab: String },
    Logs { operation: String },
    SettingsList,
    SettingsGet { key: String },
    SettingsSet { key: String, value: String },
    SettingsReset { key: Option<String> },
    RenderLabOpen,
    RenderLabFixture { id: String },
    RenderLabTopology { variant: String },
    RenderLabMotion { variant: String },
    RenderLabQuality { level: String },
    RenderLabTime { seconds: f32 },
    RenderLabStatus,
    RenderLabStyle { style: String },
    RenderLabMetrics { fixture: Option<String> },
    RenderLabRecipes,
    RenderLabRecipe { id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlResponse {
    pub version: u32,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ControlResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            ok: true,
            error: None,
            message: Some(message.into()),
            data: None,
        }
    }

    pub fn ok_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            ok: true,
            error: None,
            message: Some(message.into()),
            data: Some(data),
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            version: CONTROL_PROTOCOL_VERSION,
            ok: false,
            error: Some(error.into()),
            message: None,
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ControlStatus {
    pub protocol_version: u32,
    pub gui_running: bool,
    pub pid: u32,
    pub executable: String,
    /// The place: `packages`, `news` or `settings`.
    pub place: String,
    /// The scope of the package surface: `results`, `installed` or `updates`.
    pub scope: String,
    /// The ecosystems the scope can contain.
    pub scope_sources: Vec<String>,
    /// The ecosystems the scope lists but cannot query.
    pub inventory_only_sources: Vec<String>,
    /// The filters in force, as the interface shows them.
    pub filters: ControlFilters,
    /// The number of rows the scope shows.
    pub row_count: usize,
    /// When the scope's answer was true, when it is known.
    pub checked_at: Option<String>,
    /// The deprecated alias of `scope`, kept for the automation that reads it.
    pub destination: String,
    pub query: String,
    pub view_mode: String,
    pub inspector_tab: String,
    pub selected_package: Option<String>,
    pub operation_running: bool,
    #[serde(default)]
    pub render_lab_active: bool,
}

/// The filters in force, in the vocabulary the interface uses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ControlFilters {
    pub sources: Vec<String>,
    pub state: String,
    pub sort: String,
    pub view: String,
}

impl ControlStatus {
    pub fn not_running() -> Self {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "/usr/lib/calix/calix-gpui".to_string());
        Self {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            gui_running: false,
            pid: 0,
            executable: exe,
            place: "unknown".to_string(),
            scope: "unknown".to_string(),
            scope_sources: Vec::new(),
            inventory_only_sources: Vec::new(),
            filters: ControlFilters::default(),
            row_count: 0,
            checked_at: None,
            destination: "unknown".to_string(),
            query: String::new(),
            view_mode: "unknown".to_string(),
            inspector_tab: "unknown".to_string(),
            selected_package: None,
            operation_running: false,
            render_lab_active: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_control_protocol_round_trip() {
        let req = ControlRequest::new(ControlCommand::Navigate {
            destination: "installed".to_string(),
        });
        let json = serde_json::to_string(&req).expect("Failed to serialize request");
        let parsed: ControlRequest =
            serde_json::from_str(&json).expect("Failed to deserialize request");
        assert_eq!(req, parsed);
        assert_eq!(parsed.version, CONTROL_PROTOCOL_VERSION);

        let resp =
            ControlResponse::ok_with_data("Navigated", serde_json::json!({"dest": "installed"}));
        let resp_json = serde_json::to_string(&resp).expect("Failed to serialize response");
        let parsed_resp: ControlResponse =
            serde_json::from_str(&resp_json).expect("Failed to deserialize response");
        assert_eq!(resp, parsed_resp);
        assert!(parsed_resp.ok);
    }

    #[test]
    fn test_control_status_serialization() {
        let status = ControlStatus {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            gui_running: true,
            pid: 12345,
            executable: "/usr/lib/calix/calix-gpui".to_string(),
            place: "packages".to_string(),
            scope: "results".to_string(),
            scope_sources: vec!["alpm".to_string(), "aur".to_string()],
            inventory_only_sources: Vec::new(),
            filters: ControlFilters {
                sources: vec!["alpm".to_string()],
                state: "all".to_string(),
                sort: "relevance".to_string(),
                view: "table".to_string(),
            },
            row_count: 18,
            checked_at: None,
            destination: "results".to_string(),
            query: "ripgrep".to_string(),
            view_mode: "table".to_string(),
            inspector_tab: "overview".to_string(),
            selected_package: Some("ripgrep".to_string()),
            operation_running: false,
            render_lab_active: false,
        };
        let json = serde_json::to_string(&status).expect("Serialization failed");
        let parsed: ControlStatus = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(status, parsed);
        assert!(parsed.gui_running);
        assert_eq!(parsed.selected_package.as_deref(), Some("ripgrep"));
        assert!(!parsed.render_lab_active);
        // The place and the scope travel together, and the deprecated alias
        // still answers.
        assert_eq!(parsed.place, "packages");
        assert_eq!(parsed.scope, "results");
        assert_eq!(parsed.destination, parsed.scope);
        assert_eq!(parsed.row_count, 18);
        assert_eq!(parsed.filters.view, "table");
    }

    #[test]
    fn test_render_lab_protocol_round_trip() {
        let commands = vec![
            ControlCommand::RenderLabOpen,
            ControlCommand::RenderLabFixture {
                id: "field.signed-voltage".to_string(),
            },
            ControlCommand::RenderLabTopology {
                variant: "floating-island".to_string(),
            },
            ControlCommand::RenderLabMotion {
                variant: "smooth".to_string(),
            },
            ControlCommand::RenderLabQuality {
                level: "stock".to_string(),
            },
            ControlCommand::RenderLabTime { seconds: 0.5 },
            ControlCommand::RenderLabStatus,
            ControlCommand::RenderLabStyle {
                style: "transparency".to_string(),
            },
            ControlCommand::RenderLabMetrics {
                fixture: Some("field.zero-positive-scalar".to_string()),
            },
            ControlCommand::RenderLabMetrics { fixture: None },
            ControlCommand::RenderLabRecipes,
            ControlCommand::RenderLabRecipe {
                id: "g01-linear-scalar-gradient/r1".to_string(),
            },
        ];

        for cmd in commands {
            let req = ControlRequest::new(cmd.clone());
            let json = serde_json::to_string(&req).expect("Serialize failed");
            let deserialized: ControlRequest =
                serde_json::from_str(&json).expect("Deserialize failed");
            assert_eq!(req, deserialized);
            assert_eq!(deserialized.version, CONTROL_PROTOCOL_VERSION);
            assert_eq!(deserialized.command, cmd);
        }
    }
}
