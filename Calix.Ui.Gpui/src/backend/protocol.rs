use base64::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlpmInfoPayload {
    #[serde(rename = "EventType", default)]
    pub event_type: String,
    #[serde(rename = "Message", default)]
    pub message: String,
    #[serde(rename = "PackageName")]
    pub package_name: Option<String>,
    #[serde(rename = "CurrentIndex")]
    pub current_index: Option<usize>,
    #[serde(rename = "TotalCount")]
    pub total_count: Option<usize>,
    #[serde(rename = "Source")]
    pub source: Option<String>,
    #[serde(rename = "Level")]
    pub level: Option<String>,
    #[serde(rename = "TimeStamp")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlpmErrorPayload {
    #[serde(rename = "ErrorMessage", default)]
    pub error_message: String,
    #[serde(rename = "ErrorCode")]
    pub error_code: Option<String>,
    #[serde(rename = "Domain")]
    pub domain: Option<String>,
    #[serde(rename = "NativeCode")]
    pub native_code: Option<i64>,
    #[serde(rename = "OperationId")]
    pub operation_id: Option<u64>,
    #[serde(rename = "ParentId")]
    pub parent_id: Option<u64>,
    #[serde(rename = "Backend")]
    pub backend: Option<String>,
    #[serde(rename = "Operation")]
    pub operation: Option<String>,
    #[serde(rename = "Subject")]
    pub subject: Option<String>,
    #[serde(rename = "Recoverable")]
    pub recoverable: Option<bool>,
    #[serde(rename = "Source")]
    pub source: Option<String>,
    #[serde(rename = "Level")]
    pub level: Option<String>,
    #[serde(rename = "TimeStamp")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressPayload {
    #[serde(rename = "PackageName")]
    pub package_name: Option<String>,
    #[serde(rename = "Status")]
    pub status: Option<String>,
    #[serde(rename = "CurrentDownload")]
    pub current_download: Option<u64>,
    #[serde(rename = "TotalDownload")]
    pub total_download: Option<u64>,
    #[serde(rename = "ProgressType")]
    pub progress_type: Option<String>,
    #[serde(rename = "Percent")]
    pub percent: Option<u8>,
    #[serde(rename = "Percentage")]
    pub percentage: Option<u8>,
    #[serde(rename = "Stage")]
    pub stage: Option<String>,
    #[serde(rename = "Message")]
    pub message: Option<String>,
    #[serde(rename = "Source")]
    pub source: Option<String>,
    #[serde(rename = "Level")]
    pub level: Option<String>,
    #[serde(rename = "TimeStamp")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiFrame {
    AlpmInfo(AlpmInfoPayload),
    AlpmError(AlpmErrorPayload),
    Progress(ProgressPayload),
    GenericJson(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodedOutput {
    Frame {
        frame: Box<UiFrame>,
        human_text: String,
        is_error: bool,
    },
    MalformedFrame {
        raw: String,
        diagnostic: String,
    },
    RawText(String),
}

pub struct UiProtocolDecoder;

impl UiProtocolDecoder {
    /// Decodes a line from Calix CLI stdout/stderr.
    /// If wrapped in `[JSON]<base64>[/JSON]`, unpacks and formats into a structured event and clean human text.
    /// If not framed or if malformed, safely preserves the line without dropping data.
    pub fn decode(line: &str) -> DecodedOutput {
        let trimmed = line.trim();
        let start_tag = "[JSON]";
        let end_tag = "[/JSON]";

        let start_pos = match trimmed.find(start_tag) {
            Some(pos) => pos + start_tag.len(),
            None => return DecodedOutput::RawText(line.to_string()),
        };

        let end_pos = match trimmed[start_pos..].find(end_tag) {
            Some(pos) => start_pos + pos,
            None => return DecodedOutput::RawText(line.to_string()),
        };

        let b64_slice = trimmed[start_pos..end_pos].trim();
        let decoded_bytes = match BASE64_STANDARD.decode(b64_slice.as_bytes()) {
            Ok(bytes) => bytes,
            Err(e) => {
                return DecodedOutput::MalformedFrame {
                    raw: line.to_string(),
                    diagnostic: format!("Malformed base64: {}", e),
                };
            }
        };

        let json_val: serde_json::Value = match serde_json::from_slice(&decoded_bytes) {
            Ok(val) => val,
            Err(e) => {
                return DecodedOutput::MalformedFrame {
                    raw: line.to_string(),
                    diagnostic: format!("Malformed JSON: {}", e),
                };
            }
        };

        let kind = json_val
            .get("$kind")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match kind {
            "alpm.info" => match serde_json::from_value::<AlpmInfoPayload>(json_val) {
                Ok(info) => {
                    let is_err = info.event_type == "TransactionFailed";
                    let human = Self::format_alpm_info(&info);
                    DecodedOutput::Frame {
                        frame: Box::new(UiFrame::AlpmInfo(info)),
                        human_text: human,
                        is_error: is_err,
                    }
                }
                Err(e) => DecodedOutput::MalformedFrame {
                    raw: line.to_string(),
                    diagnostic: format!("Invalid alpm.info payload: {}", e),
                },
            },
            "alpm.error" => match serde_json::from_value::<AlpmErrorPayload>(json_val) {
                Ok(err) => {
                    let human = Self::format_alpm_error(&err);
                    DecodedOutput::Frame {
                        frame: Box::new(UiFrame::AlpmError(err)),
                        human_text: human,
                        is_error: true,
                    }
                }
                Err(e) => DecodedOutput::MalformedFrame {
                    raw: line.to_string(),
                    diagnostic: format!("Invalid alpm.error payload: {}", e),
                },
            },
            "alpm.progress" | "flatpak.progress" | "appimage.progress" => {
                match serde_json::from_value::<ProgressPayload>(json_val) {
                    Ok(prog) => {
                        let human = Self::format_progress(&prog);
                        DecodedOutput::Frame {
                            frame: Box::new(UiFrame::Progress(prog)),
                            human_text: human,
                            is_error: false,
                        }
                    }
                    Err(e) => DecodedOutput::MalformedFrame {
                        raw: line.to_string(),
                        diagnostic: format!("Invalid progress payload: {}", e),
                    },
                }
            }
            other => DecodedOutput::Frame {
                frame: Box::new(UiFrame::GenericJson(other.to_string())),
                human_text: format!("[Event: {}]", other),
                is_error: false,
            },
        }
    }

    fn format_alpm_info(info: &AlpmInfoPayload) -> String {
        match info.event_type.as_str() {
            "TransactionStart" => format!(">> Starting transaction: {}", info.message),
            "TransactionDone" => format!("✓ {}", info.message),
            "TransactionFailed" => format!("✗ {}", info.message),
            "TransactionCancelled" => format!("! Operation cancelled: {}", info.message),
            "WarningOutput" => {
                let src = info.source.as_deref().unwrap_or("System");
                format!("⚠ [{}] {}", src, info.message)
            }
            _ => {
                let src = info.source.as_deref().unwrap_or("System");
                if let Some(pkg) = &info.package_name {
                    format!("[{}] {}: {}", src, pkg, info.message)
                } else {
                    format!("[{}] {}", src, info.message)
                }
            }
        }
    }

    fn format_alpm_error(err: &AlpmErrorPayload) -> String {
        let backend = err
            .backend
            .as_deref()
            .or(err.source.as_deref())
            .unwrap_or("System");
        if let Some(code) = &err.error_code {
            format!("✗ [{}] Error ({}): {}", backend, code, err.error_message)
        } else {
            format!("✗ [{}] Error: {}", backend, err.error_message)
        }
    }

    fn format_progress(prog: &ProgressPayload) -> String {
        let percent = prog.percent.or(prog.percentage).unwrap_or(0);
        let src = prog.source.as_deref().unwrap_or("Progress");
        let desc = prog
            .status
            .as_deref()
            .or(prog.message.as_deref())
            .or(prog.stage.as_deref())
            .unwrap_or("Processing");
        if let Some(pkg) = &prog.package_name {
            format!("[{}] {}% - {}: {}", src, percent, pkg, desc)
        } else {
            format!("[{}] {}% - {}", src, percent, desc)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_ordinary_stdout_passthrough() {
        let line = "resolving dependencies...";
        let decoded = UiProtocolDecoder::decode(line);
        assert_eq!(decoded, DecodedOutput::RawText(line.to_string()));
    }

    #[test]
    fn test_decode_valid_alpm_info_frame() {
        let json = r#"{"$kind":"alpm.info","EventType":"TransactionStart","Message":"Installing 2 packages","PackageName":null,"CurrentIndex":null,"TotalCount":null,"Source":"Alpm","Level":"Information","TimeStamp":"2026-09-20T12:00:00Z"}"#;
        let b64 = BASE64_STANDARD.encode(json.as_bytes());
        let line = format!("[JSON]{}[/JSON]", b64);

        let decoded = UiProtocolDecoder::decode(&line);
        match decoded {
            DecodedOutput::Frame {
                frame,
                human_text,
                is_error,
            } => {
                assert!(!is_error);
                assert_eq!(human_text, ">> Starting transaction: Installing 2 packages");
                match *frame {
                    UiFrame::AlpmInfo(info) => {
                        assert_eq!(info.event_type, "TransactionStart");
                        assert_eq!(info.message, "Installing 2 packages");
                    }
                    _ => panic!("Expected AlpmInfo"),
                }
            }
            _ => panic!("Expected DecodedOutput::Frame"),
        }
    }

    #[test]
    fn test_decode_valid_alpm_error_frame() {
        let json = r#"{"$kind":"alpm.error","ErrorMessage":"failed to commit transaction (conflicting files)","ErrorCode":"ConflictingFiles","Domain":"alpm","NativeCode":12,"OperationId":1,"ParentId":null,"Backend":"alpm","Operation":"install","Subject":"firefox","Recoverable":false,"Source":"Alpm","Level":"Error","TimeStamp":"2026-09-20T12:00:00Z"}"#;
        let b64 = BASE64_STANDARD.encode(json.as_bytes());
        let line = format!("[JSON]{}[/JSON]", b64);

        let decoded = UiProtocolDecoder::decode(&line);
        match decoded {
            DecodedOutput::Frame {
                frame,
                human_text,
                is_error,
            } => {
                assert!(is_error);
                assert!(human_text.contains("failed to commit transaction"));
                assert!(human_text.contains("ConflictingFiles"));
                match *frame {
                    UiFrame::AlpmError(err) => {
                        assert_eq!(err.error_code, Some("ConflictingFiles".to_string()));
                        assert_eq!(err.subject, Some("firefox".to_string()));
                    }
                    _ => panic!("Expected AlpmError"),
                }
            }
            _ => panic!("Expected DecodedOutput::Frame"),
        }
    }

    #[test]
    fn test_decode_progress_frame() {
        let json = r#"{"$kind":"alpm.progress","PackageName":"linux","CurrentDownload":50,"TotalDownload":100,"ProgressType":"Download","Percent":50,"Stage":"downloading","Message":"in progress","Source":"Alpm","Level":"Information","TimeStamp":"2026-09-20T12:00:00Z"}"#;
        let b64 = BASE64_STANDARD.encode(json.as_bytes());
        let line = format!("[JSON]{}[/JSON]", b64);

        let decoded = UiProtocolDecoder::decode(&line);
        match decoded {
            DecodedOutput::Frame {
                frame,
                human_text,
                is_error,
            } => {
                assert!(!is_error);
                assert!(human_text.contains("50%"));
                assert!(human_text.contains("linux"));
                match *frame {
                    UiFrame::Progress(p) => {
                        assert_eq!(p.percent, Some(50));
                        assert_eq!(p.package_name, Some("linux".to_string()));
                    }
                    _ => panic!("Expected Progress"),
                }
            }
            _ => panic!("Expected DecodedOutput::Frame"),
        }
    }

    #[test]
    fn test_decode_malformed_base64_passthrough() {
        let line = "[JSON]invalid!!base64==[/JSON]";
        let decoded = UiProtocolDecoder::decode(line);
        match decoded {
            DecodedOutput::MalformedFrame { raw, diagnostic } => {
                assert_eq!(raw, line);
                assert!(diagnostic.contains("Malformed base64"));
            }
            _ => panic!("Expected MalformedFrame"),
        }
    }
}
