use crate::backend::protocol::{DecodedOutput, UiFrame, UiProtocolDecoder};
pub use crate::components::log_drawer::{
    BackendPhase, BackendPhaseStatus, LogEntry, OperationStatus,
};
use gpui::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleEvent {
    LogAppended(LogEntry),
    OperationStarted(String),
    OperationFinished(OperationStatus),
    Toggled(bool),
    LogsCleared,
    AutoScrollToggled(bool),
    RawLogsToggled(bool),
}

/// Modèle d'état découplé de la console d'opérations avec contrôle de défilement,
/// décodage du protocole structuré UI et suivi chronologique des backends.
pub struct ConsoleModel {
    pub logs: Vec<LogEntry>,
    pub status: OperationStatus,
    pub is_open: bool,
    pub auto_open: bool,
    pub auto_scroll: bool,
    pub show_raw_logs: bool,
    pub backend_phases: Vec<BackendPhase>,
    pub scroll_handle: ScrollHandle,
}

impl EventEmitter<ConsoleEvent> for ConsoleModel {}

impl ConsoleModel {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            status: OperationStatus::Idle,
            is_open: true,
            auto_open: true,
            auto_scroll: true,
            show_raw_logs: false,
            backend_phases: Vec::new(),
            scroll_handle: ScrollHandle::new(),
        }
    }

    pub fn set_auto_open(&mut self, auto_open: bool) {
        self.auto_open = auto_open;
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, OperationStatus::Running(_))
    }

    pub fn append_stdout(&mut self, line: &str, cx: &mut Context<Self>) {
        let decoded = UiProtocolDecoder::decode(line);
        self.handle_decoded_output(decoded, false, cx);
    }

    pub fn append_stderr(&mut self, line: &str, cx: &mut Context<Self>) {
        let decoded = UiProtocolDecoder::decode(line);
        self.handle_decoded_output(decoded, true, cx);
    }

    fn handle_decoded_output(
        &mut self,
        decoded: DecodedOutput,
        from_stderr: bool,
        cx: &mut Context<Self>,
    ) {
        let entry = match decoded {
            DecodedOutput::Frame {
                frame,
                human_text,
                is_error,
            } => {
                self.update_timeline_from_frame(&frame);
                LogEntry::decoded(human_text, is_error || from_stderr, None)
            }
            DecodedOutput::MalformedFrame { raw, diagnostic } => {
                LogEntry::decoded(format!("⚠ Protocol error: {}", diagnostic), true, Some(raw))
            }
            DecodedOutput::RawText(raw) => {
                self.infer_timeline_from_raw_text(&raw);
                LogEntry::decoded(raw.clone(), from_stderr, Some(raw))
            }
        };

        self.logs.push(entry.clone());
        if self.logs.len() > 2000 {
            self.logs.remove(0);
        }
        cx.emit(ConsoleEvent::LogAppended(entry));
        cx.notify();
    }

    fn update_timeline_from_frame(&mut self, frame: &UiFrame) {
        match frame {
            UiFrame::AlpmInfo(info) => {
                let target_backend = info.source.as_deref().unwrap_or("Standard Packages");
                let bname = match target_backend.to_lowercase().as_str() {
                    "flatpak" => "Flatpak",
                    "appimage" => "AppImage",
                    "aur" => "AUR Packages",
                    _ => "Standard Packages",
                };
                if let Some(phase) = self.backend_phases.iter_mut().find(|p| p.name == bname) {
                    if info.event_type == "TransactionDone" {
                        phase.status = BackendPhaseStatus::Success(info.message.clone());
                    } else if info.event_type == "TransactionFailed" {
                        phase.status = BackendPhaseStatus::Failed(info.message.clone());
                    } else if info.event_type == "TransactionStart" {
                        phase.status = BackendPhaseStatus::Running(None);
                    }
                }
            }
            UiFrame::AlpmError(err) => {
                let target = err
                    .backend
                    .as_deref()
                    .or(err.source.as_deref())
                    .unwrap_or("Standard Packages");
                let bname = match target.to_lowercase().as_str() {
                    "flatpak" => "Flatpak",
                    "appimage" => "AppImage",
                    "aur" => "AUR Packages",
                    _ => "Standard Packages",
                };
                if let Some(phase) = self.backend_phases.iter_mut().find(|p| p.name == bname) {
                    phase.status = BackendPhaseStatus::Failed(err.error_message.clone());
                }
            }
            UiFrame::Progress(prog) => {
                let target = prog.source.as_deref().unwrap_or("Standard Packages");
                let bname = match target.to_lowercase().as_str() {
                    "flatpak" => "Flatpak",
                    "appimage" => "AppImage",
                    "aur" => "AUR Packages",
                    _ => "Standard Packages",
                };
                if let Some(phase) = self.backend_phases.iter_mut().find(|p| p.name == bname) {
                    let pct = prog.percent.or(prog.percentage);
                    phase.status = BackendPhaseStatus::Running(pct);
                }
            }
            _ => {}
        }
    }

    fn infer_timeline_from_raw_text(&mut self, raw: &str) {
        let lower = raw.to_lowercase();
        if lower.contains("upgrading standard") || lower.contains("installing standard") {
            if let Some(phase) = self
                .backend_phases
                .iter_mut()
                .find(|p| p.name == "Standard Packages")
            {
                phase.status = BackendPhaseStatus::Running(None);
            }
        } else if lower.contains("upgrading aur") || lower.contains("building aur") {
            if let Some(phase) = self
                .backend_phases
                .iter_mut()
                .find(|p| p.name == "AUR Packages")
            {
                phase.status = BackendPhaseStatus::Running(None);
            }
        } else if lower.contains("flatpak") {
            if let Some(phase) = self.backend_phases.iter_mut().find(|p| p.name == "Flatpak") {
                phase.status = BackendPhaseStatus::Running(None);
            }
        }
    }

    /// Mutation d'état pure pour l'initialisation d'opération
    pub fn start_operation_state(&mut self, op_name: &str) -> bool {
        self.status = OperationStatus::Running(op_name.to_string());
        self.backend_phases = vec![
            BackendPhase {
                name: "Standard Packages".to_string(),
                status: BackendPhaseStatus::Pending,
            },
            BackendPhase {
                name: "AUR Packages".to_string(),
                status: BackendPhaseStatus::Pending,
            },
            BackendPhase {
                name: "Flatpak".to_string(),
                status: BackendPhaseStatus::Pending,
            },
            BackendPhase {
                name: "AppImage".to_string(),
                status: BackendPhaseStatus::Pending,
            },
        ];
        if self.auto_open {
            self.set_open_state(true)
        } else {
            false
        }
    }

    pub fn start_operation(&mut self, op_name: &str, cx: &mut Context<Self>) {
        let opened = self.start_operation_state(op_name);
        self.append_stdout(&format!(">>> Starting operation: {}", op_name), cx);
        if opened {
            cx.emit(ConsoleEvent::Toggled(true));
        }
        cx.emit(ConsoleEvent::OperationStarted(op_name.to_string()));
        cx.notify();
    }

    /// Mutation d'état pure pour la terminaison d'opération
    pub fn finish_operation_state(
        &mut self,
        success: bool,
        message: &str,
    ) -> (OperationStatus, bool) {
        let status = if success {
            OperationStatus::Success(message.to_string())
        } else {
            OperationStatus::Error(message.to_string())
        };
        self.status = status.clone();
        if success {
            for phase in &mut self.backend_phases {
                if matches!(
                    phase.status,
                    BackendPhaseStatus::Pending | BackendPhaseStatus::Running(_)
                ) {
                    phase.status = BackendPhaseStatus::Success("Complete".to_string());
                }
            }
        }
        let opened = if !success {
            self.set_open_state(true)
        } else {
            false
        };
        (status, opened)
    }

    pub fn finish_operation(&mut self, success: bool, message: &str, cx: &mut Context<Self>) {
        let (status, opened) = self.finish_operation_state(success, message);
        let log_line = if success {
            format!(">>> Success: {}", message)
        } else {
            format!(">>> Failed: {}", message)
        };
        if success {
            self.append_stdout(&log_line, cx);
        } else {
            self.append_stderr(&log_line, cx);
        }
        if opened {
            cx.emit(ConsoleEvent::Toggled(true));
        }
        cx.emit(ConsoleEvent::OperationFinished(status));
        cx.notify();
    }

    pub fn set_open_state(&mut self, open: bool) -> bool {
        if self.is_open != open {
            self.is_open = open;
            true
        } else {
            false
        }
    }

    pub fn toggle_drawer(&mut self, cx: &mut Context<Self>) {
        let new_state = !self.is_open;
        self.set_open(new_state, cx);
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.set_open_state(open) {
            cx.emit(ConsoleEvent::Toggled(open));
            cx.notify();
        }
    }

    pub fn toggle_auto_scroll(&mut self, cx: &mut Context<Self>) {
        self.auto_scroll = !self.auto_scroll;
        cx.emit(ConsoleEvent::AutoScrollToggled(self.auto_scroll));
        cx.notify();
    }

    pub fn toggle_raw_logs(&mut self, cx: &mut Context<Self>) {
        self.show_raw_logs = !self.show_raw_logs;
        cx.emit(ConsoleEvent::RawLogsToggled(self.show_raw_logs));
        cx.notify();
    }

    pub fn copy_text(&self) -> String {
        let mut out = String::new();
        if !self.backend_phases.is_empty() {
            out.push_str("=== OPERATION TIMELINE ===\n");
            for p in &self.backend_phases {
                let st = match &p.status {
                    BackendPhaseStatus::Pending => "Pending".to_string(),
                    BackendPhaseStatus::Running(pct) => {
                        if let Some(p) = pct {
                            format!("Running ({}%)", p)
                        } else {
                            "Running".to_string()
                        }
                    }
                    BackendPhaseStatus::Success(s) => format!("✓ {}", s),
                    BackendPhaseStatus::Failed(f) => format!("✕ {}", f),
                };
                out.push_str(&format!("{:<20} : {}\n", p.name, st));
            }
            out.push_str("==========================\n\n");
        }
        for entry in &self.logs {
            if self.show_raw_logs {
                if let Some(raw) = &entry.raw {
                    out.push_str(raw);
                } else {
                    out.push_str(&entry.text);
                }
            } else {
                out.push_str(&entry.text);
            }
            out.push('\n');
        }
        out
    }

    /// Efface l'historique des lignes de log affichées sans altérer le statut du cycle de vie
    pub fn clear_history(&mut self) {
        self.logs.clear();
    }

    pub fn clear_logs(&mut self, cx: &mut Context<Self>) {
        self.clear_history();
        cx.emit(ConsoleEvent::LogsCleared);
        cx.notify();
    }
}

impl Default for ConsoleModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::prelude::*;
    use core::prelude::v1::test;

    #[test]
    fn test_console_model_defaults() {
        let console = ConsoleModel::new();
        assert!(console.logs.is_empty());
        assert_eq!(console.status, OperationStatus::Idle);
        assert!(console.is_open);
        assert!(console.auto_open);
        assert!(console.auto_scroll);
        assert!(!console.show_raw_logs);
        assert!(console.backend_phases.is_empty());
    }

    #[test]
    fn test_clear_logs_preserves_lifecycle_status() {
        let mut console = ConsoleModel::new();
        console.status = OperationStatus::Running("upgrade".to_string());
        console
            .logs
            .push(LogEntry::decoded("Building package...", false, None));

        assert_eq!(console.logs.len(), 1);
        assert_eq!(
            console.status,
            OperationStatus::Running("upgrade".to_string())
        );

        console.clear_history();
        assert!(console.logs.is_empty());
        assert_eq!(
            console.status,
            OperationStatus::Running("upgrade".to_string())
        );

        console.status = OperationStatus::Success("Done".to_string());
        console
            .logs
            .push(LogEntry::decoded("Finished successfully", false, None));
        console.clear_history();
        assert!(console.logs.is_empty());
        assert_eq!(console.status, OperationStatus::Success("Done".to_string()));
    }

    #[test]
    fn test_start_operation_honors_auto_open_setting() {
        let mut console = ConsoleModel::new();
        console.is_open = false;

        console.auto_open = false;
        let opened = console.start_operation_state("pkg-build");
        assert!(!opened);
        assert!(
            !console.is_open,
            "Drawer must remain closed when auto_open is disabled"
        );
        assert_eq!(
            console.status,
            OperationStatus::Running("pkg-build".to_string())
        );

        console.is_open = false;
        console.auto_open = true;
        let opened = console.start_operation_state("pkg-install");
        assert!(opened);
        assert!(
            console.is_open,
            "Drawer must be logically open when auto_open is enabled"
        );
    }

    #[test]
    fn test_finish_operation_failure_auto_discloses_logical_open() {
        let mut console = ConsoleModel::new();
        console.is_open = false;
        console.status = OperationStatus::Running("download".to_string());

        let (status, opened) = console.finish_operation_state(false, "Network failure");
        assert!(opened, "Failure must trigger logical disclosure");
        assert!(
            console.is_open,
            "ConsoleModel.is_open must be true on failure"
        );
        assert_eq!(
            status,
            OperationStatus::Error("Network failure".to_string())
        );

        console.is_open = false;
        let (status, opened) = console.finish_operation_state(true, "All packages up to date");
        assert!(!opened);
        assert!(!console.is_open);
        assert_eq!(
            status,
            OperationStatus::Success("All packages up to date".to_string())
        );
    }

    #[test]
    fn test_console_is_running_authority() {
        let mut console = ConsoleModel::new();
        assert!(!console.is_running());

        console.start_operation_state("upgrade");
        assert!(console.is_running());

        console.finish_operation_state(true, "Upgrade complete");
        assert!(!console.is_running());

        console.start_operation_state("install");
        assert!(console.is_running());

        console.finish_operation_state(false, "Failed");
        assert!(!console.is_running());
    }

    #[test]
    fn test_timeline_phases_initialized_and_updated() {
        let mut console = ConsoleModel::new();
        console.start_operation_state("System Upgrade");
        assert_eq!(console.backend_phases.len(), 4);
        assert_eq!(
            console.backend_phases[0].status,
            BackendPhaseStatus::Pending
        );

        let json = r#"{"$kind":"alpm.info","EventType":"TransactionDone","Message":"Standard packages up to date","PackageName":null,"Source":"Alpm","Level":"Information"}"#;
        let b64 = BASE64_STANDARD.encode(json.as_bytes());
        let frame = UiProtocolDecoder::decode(&format!("[JSON]{}[/JSON]", b64));

        if let DecodedOutput::Frame { frame, .. } = frame {
            console.update_timeline_from_frame(&frame);
        }

        assert_eq!(
            console.backend_phases[0].status,
            BackendPhaseStatus::Success("Standard packages up to date".to_string())
        );

        let copy = console.copy_text();
        assert!(copy.contains("=== OPERATION TIMELINE ==="));
        assert!(copy.contains("Standard Packages"));
        assert!(copy.contains("Standard packages up to date"));
    }
}
