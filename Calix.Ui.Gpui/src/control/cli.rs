use crate::config::ConfigManager;
use crate::control::protocol::{
    ControlCommand, ControlResponse, ControlStatus, CONTROL_PROTOCOL_VERSION,
};
use crate::control::socket::ControlSocket;
use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct CliInvocation {
    pub json: bool,
    pub command: Option<CliCommand>,
    pub internal_gui: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    Help,
    Version,
    Open,
    Focus,
    Status,
    Quit,
    Navigate { destination: String },
    /// Change the scope of the package surface, without leaving the place.
    Scope { scope: String },
    Search { query: String },
    View { mode: String },
    Inspect { package: String },
    Inspector { tab: String },
    Logs { operation: String },
    Settings(CliSettingsCommand),
    RenderLab(CliRenderLabCommand),
    Appearance(CliAppearanceCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliRenderLabCommand {
    Open,
    Fixture { id: String },
    Material { id: String },
    Topology { variant: String },
    Motion { variant: String },
    Quality { level: String },
    Time { seconds: f32 },
    Status,
    Style { style: String },
    Ledger,
    Metrics { fixture: Option<String> },
    Recipes,
    Recipe { id: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliSettingsCommand {
    List,
    Get { key: String },
    Set { key: String, value: String },
    Reset { key: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CliAppearanceCommand {
    StyleGet,
    StyleSet { value: String },
    Status,
    Resolve { role: Option<String> },
}

pub enum CliOutcome {
    LaunchGui,
    Exit(i32),
}

impl CliInvocation {
    pub fn parse_from_args(args: &[String]) -> Result<Self, String> {
        if args.iter().any(|a| a == "--internal-gui") {
            return Ok(Self {
                json: false,
                command: None,
                internal_gui: true,
            });
        }

        let mut json = false;
        let mut positional = Vec::new();

        for arg in args.iter().skip(1) {
            match arg.as_str() {
                "-j" | "--json" => json = true,
                "-h" | "--help" => {
                    return Ok(Self {
                        json,
                        command: Some(CliCommand::Help),
                        internal_gui: false,
                    })
                }
                "-V" | "--version" => {
                    return Ok(Self {
                        json,
                        command: Some(CliCommand::Version),
                        internal_gui: false,
                    })
                }
                other => positional.push(other.to_string()),
            }
        }

        if positional.is_empty() {
            return Ok(Self {
                json,
                command: None,
                internal_gui: false,
            });
        }

        let cmd_str = positional[0].to_ascii_lowercase();
        let cmd = match cmd_str.as_str() {
            "help" => CliCommand::Help,
            "version" => CliCommand::Version,
            "open" => CliCommand::Open,
            "focus" => CliCommand::Focus,
            "status" => CliCommand::Status,
            "quit" => CliCommand::Quit,
            "navigate" => {
                if positional.len() < 2 {
                    return Err("Missing destination argument for 'navigate' (e.g. updates, installed, browse)".into());
                }
                CliCommand::Navigate {
                    destination: positional[1].clone(),
                }
            }
            "scope" => {
                if positional.len() < 2 {
                    return Err(
                        "Missing scope argument for 'scope' (e.g. results, installed, updates)"
                            .into(),
                    );
                }
                CliCommand::Scope {
                    scope: positional[1].clone(),
                }
            }
            "search" => {
                if positional.len() < 2 {
                    return Err("Missing query argument for 'search'".into());
                }
                let query = positional[1..].join(" ");
                CliCommand::Search { query }
            }
            "view" => {
                if positional.len() < 2 {
                    return Err("Missing view mode argument for 'view' ('table' or 'cards')".into());
                }
                let mode = positional[1].to_ascii_lowercase();
                if mode != "table" && mode != "cards" {
                    return Err(format!(
                        "Invalid view mode '{mode}', expected 'table' or 'cards'"
                    ));
                }
                CliCommand::View { mode }
            }
            "inspect" => {
                if positional.len() < 2 {
                    return Err("Missing package argument for 'inspect'".into());
                }
                CliCommand::Inspect {
                    package: positional[1].clone(),
                }
            }
            "inspector" => {
                if positional.len() < 2 {
                    return Err("Missing tab argument for 'inspector' ('overview', 'dependencies', 'files')".into());
                }
                CliCommand::Inspector {
                    tab: positional[1].clone(),
                }
            }
            "logs" => {
                if positional.len() < 2 {
                    return Err(
                        "Missing operation argument for 'logs' ('show', 'hide', 'clear')".into(),
                    );
                }
                let op = positional[1].to_ascii_lowercase();
                if op != "show" && op != "hide" && op != "clear" {
                    return Err(format!(
                        "Invalid logs operation '{op}', expected 'show', 'hide', or 'clear'"
                    ));
                }
                CliCommand::Logs { operation: op }
            }
            "settings" => {
                if positional.len() < 2 {
                    return Err(
                        "Missing settings subcommand ('list', 'get', 'set', 'reset')".into(),
                    );
                }
                let sub = positional[1].to_ascii_lowercase();
                match sub.as_str() {
                    "list" => CliCommand::Settings(CliSettingsCommand::List),
                    "get" => {
                        if positional.len() < 3 {
                            return Err("Missing setting key for 'settings get'".into());
                        }
                        CliCommand::Settings(CliSettingsCommand::Get { key: positional[2].clone() })
                    }
                    "set" => {
                        if positional.len() < 3 {
                            return Err("Missing setting key for 'settings set'".into());
                        }
                        if positional.len() < 4 {
                            return Err("Missing setting value for 'settings set'".into());
                        }
                        CliCommand::Settings(CliSettingsCommand::Set {
                            key: positional[2].clone(),
                            value: positional[3].clone(),
                        })
                    }
                    "reset" => {
                        let key = if positional.len() >= 3 {
                            let k = positional[2].clone();
                            if k.eq_ignore_ascii_case("all") {
                                None
                            } else {
                                Some(k)
                            }
                        } else {
                            None
                        };
                        CliCommand::Settings(CliSettingsCommand::Reset { key })
                    }
                    other => return Err(format!("Unknown settings subcommand '{other}'. Expected 'list', 'get', 'set', or 'reset'")),
                }
            }
            "render-lab" | "render_lab" => {
                if positional.len() < 2 {
                    return Err(
                        "Missing subcommand for 'render-lab' ('open', 'fixture', 'material', 'topology', 'motion', 'quality', 'time', 'status', 'style', 'ledger', 'metrics', 'recipes', 'recipe')".into(),
                    );
                }
                let sub = positional[1].to_ascii_lowercase();
                match sub.as_str() {
                    "open" => CliCommand::RenderLab(CliRenderLabCommand::Open),
                    "fixture" => {
                        if positional.len() < 3 {
                            return Err("Missing fixture ID for 'render-lab fixture'".into());
                        }
                        CliCommand::RenderLab(CliRenderLabCommand::Fixture {
                            id: positional[2].clone(),
                        })
                    }
                    "material" => {
                        if positional.len() < 3 {
                            return Err("Missing material name for 'render-lab material'".into());
                        }
                        CliCommand::RenderLab(CliRenderLabCommand::Material {
                            id: positional[2].clone(),
                        })
                    }
                    "topology" => {
                        if positional.len() < 3 {
                            return Err(
                                "Missing variant for 'render-lab topology' ('floating-island', 'full-band', 'perimeter-hug')".into(),
                            );
                        }
                        let variant = positional[2].to_ascii_lowercase();
                        crate::render_lab::TopologyVariant::parse(&variant)?;
                        CliCommand::RenderLab(CliRenderLabCommand::Topology { variant })
                    }
                    "motion" => {
                        if positional.len() < 3 {
                            return Err(
                                "Missing variant for 'render-lab motion' ('classic', 'smooth', 'elastic', 'liquid', 'reduced-motion')".into(),
                            );
                        }
                        let variant = positional[2].to_ascii_lowercase();
                        crate::render_lab::MotionVariant::parse(&variant)?;
                        CliCommand::RenderLab(CliRenderLabCommand::Motion { variant })
                    }
                    "quality" => {
                        if positional.len() < 3 {
                            return Err(
                                "Missing level for 'render-lab quality' ('stock')".into(),
                            );
                        }
                        let level = positional[2].to_ascii_lowercase();
                        crate::render_lab::QualityLevel::parse(&level)?;
                        CliCommand::RenderLab(CliRenderLabCommand::Quality { level })
                    }
                    "time" => {
                        if positional.len() < 3 {
                            return Err("Missing time in seconds for 'render-lab time'".into());
                        }
                        let seconds: f32 = positional[2].parse().map_err(|_| {
                            format!(
                                "Invalid time value '{}', expected a non-negative number",
                                positional[2]
                            )
                        })?;
                        if seconds < 0.0 || seconds.is_nan() || seconds.is_infinite() {
                            return Err(format!(
                                "Invalid time value '{seconds}', expected non-negative finite number"
                            ));
                        }
                        CliCommand::RenderLab(CliRenderLabCommand::Time { seconds })
                    }
                    "status" => CliCommand::RenderLab(CliRenderLabCommand::Status),
                    "style" => {
                        if positional.len() < 3 {
                            return Err(
                                "Missing style for 'render-lab style' ('standard' or 'transparency')".into(),
                            );
                        }
                        let style = positional[2].to_ascii_lowercase();
                        crate::visual_style::VisualStyleId::parse(&style)?;
                        CliCommand::RenderLab(CliRenderLabCommand::Style { style })
                    }
                    "ledger" => CliCommand::RenderLab(CliRenderLabCommand::Ledger),
                    "metrics" => {
                        let fixture = if positional.len() >= 3 {
                            Some(positional[2].clone())
                        } else {
                            None
                        };
                        CliCommand::RenderLab(CliRenderLabCommand::Metrics { fixture })
                    }
                    "recipes" => CliCommand::RenderLab(CliRenderLabCommand::Recipes),
                    "recipe" => {
                        if positional.len() < 3 {
                            return Err(
                                "Missing recipe or fixture ID for 'render-lab recipe'".into(),
                            );
                        }
                        CliCommand::RenderLab(CliRenderLabCommand::Recipe {
                            id: positional[2].clone(),
                        })
                    }
                    other => {
                        return Err(format!(
                            "Unknown render-lab subcommand '{other}'. Expected 'open', 'fixture', 'material', 'topology', 'motion', 'quality', 'time', 'status', 'style', 'ledger', 'metrics', 'recipes', or 'recipe'"
                        ))
                    }
                }
            }
            "appearance" => {
                if positional.len() < 2 {
                    return Err(
                        "Missing subcommand for 'appearance' ('style', 'status', 'resolve')".into(),
                    );
                }
                let sub = positional[1].to_ascii_lowercase();
                match sub.as_str() {
                    "style" => {
                        if positional.len() < 3 {
                            return Err(
                                "Missing operation for 'appearance style' ('get' or 'set')".into(),
                            );
                        }
                        let op = positional[2].to_ascii_lowercase();
                        match op.as_str() {
                            "get" => CliCommand::Appearance(CliAppearanceCommand::StyleGet),
                            "set" => {
                                if positional.len() < 4 {
                                    return Err("Missing value for 'appearance style set' ('standard' or 'transparency')".into());
                                }
                                let value = positional[3].to_ascii_lowercase();
                                crate::visual_style::VisualStyleId::parse(&value)?;
                                CliCommand::Appearance(CliAppearanceCommand::StyleSet { value })
                            }
                            other => {
                                return Err(format!(
                                    "Unknown appearance style operation '{other}'. Expected 'get' or 'set'"
                                ))
                            }
                        }
                    }
                    "status" => CliCommand::Appearance(CliAppearanceCommand::Status),
                    "resolve" => {
                        let role = if positional.len() >= 3 {
                            let role_str = positional[2].to_ascii_lowercase();
                            crate::visual_style::SurfaceRole::parse(&role_str)?;
                            Some(role_str)
                        } else {
                            None
                        };
                        CliCommand::Appearance(CliAppearanceCommand::Resolve { role })
                    }
                    other => {
                        return Err(format!(
                            "Unknown appearance subcommand '{other}'. Expected 'style', 'status', or 'resolve'"
                        ))
                    }
                }
            }
            unknown => {
                return Err(format!(
                    "Unknown command '{unknown}'. Run 'calix-gpui --help' for usage."
                ))
            }
        };

        Ok(Self {
            json,
            command: Some(cmd),
            internal_gui: false,
        })
    }

    pub fn print_help() {
        println!(
            r#"Calix GPUI — GPU-Accelerated Universal Package Manager
Usage: calix-gpui [OPTIONS] [SUBCOMMAND]

Options:
  -j, --json     Output machine-readable JSON
  -h, --help     Print help and exit
  -V, --version  Print version and exit

Runtime Control Commands:
  open                       Focus existing Calix window, or launch GUI if offline
  focus                      Focus running Calix window
  status                     Print runtime status and health
  quit                       Cleanly terminate running Calix instance
  navigate <dest>            Navigate to 'updates', 'installed', 'browse', 'news', 'settings'
  scope <scope>              Change the package scope: 'results', 'installed', 'updates'
  search <query>             Search for packages in the browse workstation
  view <table|cards>         Switch package workstation view mode
  inspect <package>          Select and inspect a package
  inspector <tab>            Switch inspector tab ('overview', 'dependencies', 'files')
  logs <show|hide|clear>     Show, hide, or clear the operation console log drawer

Settings Authority Commands:
  settings list              List all supported settings, defaults, and authorities
  settings get <key>         Get current value of a setting
  settings set <key> <val>   Set and validate a setting value atomically
  settings reset [key|all]   Reset a specific setting or all settings to defaults

Visual Style Authority Commands (STYLE-00A):
  appearance style get                 Get the active visual style profile
  appearance style set <style>         Set 'standard' or 'transparency' (live when GUI runs)
  appearance status                    Report style, color scheme and resolved effects
  appearance resolve [role]            Resolve a surface role projection (lists roles if omitted)

Render Lab Commands (RENDER-00, CLI-only developer surface):
  render-lab open                      Open the Render Lab destination (launches GUI if needed)
  render-lab fixture <id>              Select the active fixture by ID
  render-lab material <name>           Select a material fixture by short name
  render-lab topology <variant>        Set 'floating-island', 'full-band' or 'perimeter-hug'
  render-lab motion <variant>          Set 'classic', 'smooth', 'elastic', 'liquid' or 'reduced-motion'
  render-lab quality <level>           Set quality level ('stock')
  render-lab time <seconds>            Freeze the lab clock at t seconds
  render-lab status                    Print the lab manifest (works offline)
  render-lab style <style>            Set lab style axis ('standard' or 'transparency')
  render-lab ledger                    Print the capability ledger (works offline)
  render-lab metrics [fixture]         Print structural metrics table or fixture metrics (works offline)
  render-lab recipes                   List all 46 canonical recipe plans (works offline)
  render-lab recipe <id>               Print details and plan for a specific recipe (works offline)
"#
        );
    }

    pub fn print_version() {
        println!(
            "calix-gpui {} (control protocol v{})",
            env!("CARGO_PKG_VERSION"),
            CONTROL_PROTOCOL_VERSION
        );
    }
}

/// Ensures that a background Calix GPUI instance is running and listening on the socket.
/// Spawns the GUI in detached mode with `--internal-gui` and waits bounded time for socket readiness.
pub async fn ensure_gui_running() -> Result<()> {
    if ControlSocket::is_instance_running().await {
        return Ok(());
    }

    use std::os::unix::process::CommandExt;
    let exe = std::env::current_exe()?;
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("--internal-gui")
        .process_group(0)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    if std::env::var("CALIX_BIN").is_err()
        && std::path::Path::new("/usr/lib/calix/calix").exists()
    {
        cmd.env("CALIX_BIN", "/usr/lib/calix/calix");
    }

    let _child = cmd.spawn()?;

    // Wait bounded readiness for the socket (up to 3 seconds, polling every 50ms)
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if ControlSocket::is_instance_running().await {
            return Ok(());
        }
    }

    anyhow::bail!("Timed out waiting for Calix GPUI instance to initialize socket")
}

pub async fn run_cli_invocation(invocation: CliInvocation) -> Result<CliOutcome> {
    if invocation.internal_gui {
        return Ok(CliOutcome::LaunchGui);
    }

    let json = invocation.json;

    let cmd = match invocation.command {
        Some(CliCommand::Help) => {
            CliInvocation::print_help();
            return Ok(CliOutcome::Exit(0));
        }
        Some(CliCommand::Version) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "version": env!("CARGO_PKG_VERSION"),
                        "control_protocol": CONTROL_PROTOCOL_VERSION,
                    })
                );
            } else {
                CliInvocation::print_version();
            }
            return Ok(CliOutcome::Exit(0));
        }
        None => {
            // Check if instance is running
            if ControlSocket::is_instance_running().await {
                // Focus existing window
                let resp = ControlSocket::send_command(ControlCommand::Open).await?;
                return outcome_from_response(&resp, json);
            } else {
                return Ok(CliOutcome::LaunchGui);
            }
        }
        Some(c) => c,
    };

    let is_running = ControlSocket::is_instance_running().await;

    match cmd {
        CliCommand::Help | CliCommand::Version => unreachable!(),
        CliCommand::Open => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Open).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Focus => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Focus).await?;
                outcome_from_response(&resp, json)
            } else {
                let resp = ControlResponse::error("Calix GUI is not running");
                outcome_from_response(&resp, json)
            }
        }
        CliCommand::Status => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Status).await?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else if let Some(ref data) = resp.data {
                    if let Ok(st) = serde_json::from_value::<ControlStatus>(data.clone()) {
                        println!("Calix GPUI Status:");
                        println!("  Running:        yes (PID {})", st.pid);
                        println!("  Executable:     {}", st.executable);
                        println!("  Protocol:       v{}", st.protocol_version);
                        println!("  Destination:    {}", st.destination);
                        println!("  Search Query:   \"{}\"", st.query);
                        println!("  View Mode:      {}", st.view_mode);
                        println!("  Inspector Tab:  {}", st.inspector_tab);
                        println!(
                            "  Selected Pkg:   {}",
                            st.selected_package.as_deref().unwrap_or("<none>")
                        );
                        println!("  Active Task:    {}", st.operation_running);
                    } else {
                        println!("Calix GUI is running (status data available via --json)");
                    }
                } else {
                    println!("Calix GUI is running");
                }
                Ok(CliOutcome::Exit(if resp.ok { 0 } else { 1 }))
            } else {
                let offline_status = ControlStatus::not_running();
                if json {
                    let resp = ControlResponse::ok_with_data(
                        "Offline",
                        serde_json::to_value(&offline_status).unwrap_or_default(),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!("Calix GPUI Status:");
                    println!("  Running:        no (offline)");
                    println!("  Executable:     {}", offline_status.executable);
                    println!("  Protocol:       v{}", offline_status.protocol_version);
                }
                Ok(CliOutcome::Exit(0))
            }
        }
        CliCommand::Quit => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Quit).await?;
                outcome_from_response(&resp, json)
            } else {
                let resp = ControlResponse::error("Calix GUI is not running");
                outcome_from_response(&resp, json)
            }
        }
        CliCommand::Navigate { destination } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp =
                ControlSocket::send_command(ControlCommand::Navigate { destination }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Scope { scope } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Scope { scope }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Search { query } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Search { query }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::View { mode } => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::View { mode }).await?;
                outcome_from_response(&resp, json)
            } else {
                match ConfigManager::set_setting("view-mode", &mode) {
                    Ok(()) => {
                        let resp =
                            ControlResponse::ok(format!("View mode set to '{mode}' (offline)"));
                        outcome_from_response(&resp, json)
                    }
                    Err(e) => {
                        let resp = ControlResponse::error(e.to_string());
                        outcome_from_response(&resp, json)
                    }
                }
            }
        }
        CliCommand::Inspect { package } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Inspect { package }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Inspector { tab } => {
            if !is_running {
                ensure_gui_running().await?;
            }
            let resp = ControlSocket::send_command(ControlCommand::Inspector { tab }).await?;
            outcome_from_response(&resp, json)
        }
        CliCommand::Logs { operation } => {
            if is_running {
                let resp = ControlSocket::send_command(ControlCommand::Logs { operation }).await?;
                outcome_from_response(&resp, json)
            } else if operation == "show" || operation == "hide" {
                let open_val = if operation == "show" { "true" } else { "false" };
                let _ = ConfigManager::set_setting("log-drawer-open", open_val);
                let resp = ControlResponse::ok(format!(
                    "Log drawer default set to '{operation}' (offline)"
                ));
                outcome_from_response(&resp, json)
            } else {
                let resp = ControlResponse::error(
                    "Calix GUI is not running (cannot clear in-memory logs)",
                );
                outcome_from_response(&resp, json)
            }
        }
        CliCommand::Settings(settings_cmd) => match settings_cmd {
            CliSettingsCommand::List => {
                let entries = ConfigManager::list_settings();
                if json {
                    let resp = ControlResponse::ok_with_data(
                        "Settings listed",
                        serde_json::to_value(&entries).unwrap_or_default(),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!(
                        "{:<22} {:<12} {:<10} {:<10} DESCRIPTION",
                        "KEY", "VALUE", "DEFAULT", "AUTHORITY"
                    );
                    println!("{}", "-".repeat(88));
                    for e in entries {
                        println!(
                            "{:<22} {:<12} {:<10} {:<10} {}",
                            e.key, e.value, e.default, e.authority, e.description
                        );
                    }
                }
                Ok(CliOutcome::Exit(0))
            }
            CliSettingsCommand::Get { key } => match ConfigManager::get_setting(&key) {
                Ok(val) => {
                    if json {
                        let resp = ControlResponse::ok_with_data(
                            format!("{key}: {val}"),
                            serde_json::json!({ "key": key, "value": val }),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!("{key} = {val}");
                    }
                    Ok(CliOutcome::Exit(0))
                }
                Err(e) => {
                    let resp = ControlResponse::error(e.to_string());
                    outcome_from_response(&resp, json)
                }
            },
            CliSettingsCommand::Set { key, value } => {
                if is_running {
                    let resp =
                        ControlSocket::send_command(ControlCommand::SettingsSet { key, value })
                            .await?;
                    outcome_from_response(&resp, json)
                } else {
                    match ConfigManager::set_setting(&key, &value) {
                        Ok(()) => {
                            let resp = ControlResponse::ok(format!(
                                "Setting '{key}' set to '{value}' (offline)"
                            ));
                            outcome_from_response(&resp, json)
                        }
                        Err(e) => {
                            let resp = ControlResponse::error(e.to_string());
                            outcome_from_response(&resp, json)
                        }
                    }
                }
            }
            CliSettingsCommand::Reset { key } => {
                if is_running {
                    let resp =
                        ControlSocket::send_command(ControlCommand::SettingsReset { key }).await?;
                    outcome_from_response(&resp, json)
                } else {
                    let res = match key.as_deref() {
                        Some(k) => ConfigManager::reset_setting(k),
                        None => ConfigManager::reset_all(),
                    };
                    match res {
                        Ok(()) => {
                            let target = key.as_deref().unwrap_or("all settings");
                            let resp = ControlResponse::ok(format!(
                                "Reset '{target}' to default (offline)"
                            ));
                            outcome_from_response(&resp, json)
                        }
                        Err(e) => {
                            let resp = ControlResponse::error(e.to_string());
                            outcome_from_response(&resp, json)
                        }
                    }
                }
            }
        },
        CliCommand::RenderLab(lab_cmd) => match lab_cmd {
            CliRenderLabCommand::Open => {
                ensure_gui_running().await?;
                let resp = ControlSocket::send_command(ControlCommand::RenderLabOpen).await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Fixture { id } => {
                ensure_gui_running().await?;
                let resp =
                    ControlSocket::send_command(ControlCommand::RenderLabFixture { id }).await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Material { id } => {
                ensure_gui_running().await?;
                let fixture_id = if id.starts_with("material.") {
                    id
                } else {
                    format!("material.{id}")
                };
                let resp = ControlSocket::send_command(ControlCommand::RenderLabFixture {
                    id: fixture_id,
                })
                .await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Topology { variant } => {
                ensure_gui_running().await?;
                let resp =
                    ControlSocket::send_command(ControlCommand::RenderLabTopology { variant })
                        .await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Motion { variant } => {
                ensure_gui_running().await?;
                let resp = ControlSocket::send_command(ControlCommand::RenderLabMotion { variant })
                    .await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Quality { level } => {
                ensure_gui_running().await?;
                let resp =
                    ControlSocket::send_command(ControlCommand::RenderLabQuality { level }).await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Time { seconds } => {
                ensure_gui_running().await?;
                let resp =
                    ControlSocket::send_command(ControlCommand::RenderLabTime { seconds }).await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Status => {
                if is_running {
                    let resp = ControlSocket::send_command(ControlCommand::RenderLabStatus).await?;
                    outcome_from_response(&resp, json)
                } else {
                    let manifest = crate::render_lab::RenderLabManifest::offline();
                    if json {
                        let resp = ControlResponse::ok_with_data(
                            "Offline",
                            serde_json::to_value(&manifest).unwrap_or_default(),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!("Render Lab Status (Offline)");
                        println!("Catalog Count: {}", manifest.catalog_count);
                        println!("Clock Mode:    {}", manifest.clock_mode);
                        println!("Topology:      {}", manifest.topology);
                        println!("Motion:        {}", manifest.motion);
                        println!("Quality:       {}", manifest.quality);
                        println!("Style:         {}", manifest.style);
                    }
                    Ok(CliOutcome::Exit(0))
                }
            }
            CliRenderLabCommand::Style { style } => {
                ensure_gui_running().await?;
                let resp =
                    ControlSocket::send_command(ControlCommand::RenderLabStyle { style }).await?;
                outcome_from_response(&resp, json)
            }
            CliRenderLabCommand::Ledger => {
                if let Err(e) = crate::render_lab::ledger::CapabilityLedger::validate() {
                    let resp = ControlResponse::error(format!("Ledger invalid: {e}"));
                    return outcome_from_response(&resp, json);
                }
                let observations = crate::render_lab::ledger::CapabilityLedger::all();
                if json {
                    let resp = ControlResponse::ok_with_data(
                        "Capability ledger",
                        serde_json::to_value(observations).unwrap_or_default(),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!(
                        "Capability Ledger ({} observations, backend {}):",
                        observations.len(),
                        crate::render_lab::ledger::STOCK_BACKEND_ID
                    );
                    for obs in observations {
                        println!(
                            "  {:<34} {:<13} {}  [{}]",
                            obs.fixture, obs.verdict, obs.recipe, obs.evidence
                        );
                    }
                }
                Ok(CliOutcome::Exit(0))
            }
            CliRenderLabCommand::Metrics { fixture } => {
                if let Some(fid) = fixture {
                    let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                    let fdef = catalog.iter().find(|d| d.id.as_str() == fid.as_str());
                    let (recipe_id, fixture_str) = if let Some(def) = fdef {
                        if let Some(r) =
                            crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                        {
                            (r.id.to_string(), def.id.as_str().to_string())
                        } else {
                            let resp = ControlResponse::error(format!(
                                "No recipe for fixture '{}'",
                                def.id.as_str()
                            ));
                            return outcome_from_response(&resp, json);
                        }
                    } else {
                        (fid.clone(), String::new())
                    };

                    let plan = match crate::render_lab::plans::plan_for_recipe(&recipe_id) {
                        Some(p) => p,
                        None => {
                            let resp = ControlResponse::error(format!(
                                "Recipe plan not found for '{recipe_id}'"
                            ));
                            return outcome_from_response(&resp, json);
                        }
                    };

                    let metrics = plan.structural_metrics();
                    if json {
                        let resp = ControlResponse::ok_with_data(
                            format!("Structural metrics for {recipe_id}"),
                            serde_json::json!({
                                "fixture": if fixture_str.is_empty() { None } else { Some(&fixture_str) },
                                "recipe": recipe_id,
                                "metrics": metrics,
                            }),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!("Structural Metrics for {}:", recipe_id);
                        if !fixture_str.is_empty() {
                            println!("  Fixture:            {}", fixture_str);
                        }
                        println!("  Graph Nodes:        {}", metrics.graph_nodes);
                        println!("  Expanded Ops:       {}", metrics.expanded_ops);
                        println!("  Max Nesting Depth:  {}", metrics.max_depth);
                        println!("  Fill Operations:    {}", metrics.fill_ops);
                        println!("  Linear Gradients:   {}", metrics.gradient_ops);
                        println!("  Borders:            {}", metrics.border_ops);
                        println!("  Shadow Lobes:       {}", metrics.shadow_lobes);
                        println!(
                            "  Textures:           {} ({} px, {} bytes)",
                            metrics.texture_count,
                            metrics.texture_pixels,
                            metrics.texture_rgba_bytes
                        );
                    }
                    Ok(CliOutcome::Exit(0))
                } else {
                    let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                    let mut rows = Vec::new();
                    for def in catalog {
                        if let Some(recipe) =
                            crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                        {
                            if let Some(plan) = crate::render_lab::plans::plan_for_recipe(recipe.id)
                            {
                                let m = plan.structural_metrics();
                                rows.push((def.id.as_str(), recipe.id, m));
                            }
                        }
                    }

                    if json {
                        let items: Vec<serde_json::Value> = rows
                            .iter()
                            .map(|(f, r, m)| {
                                serde_json::json!({
                                    "fixture": f,
                                    "recipe": r,
                                    "metrics": m,
                                })
                            })
                            .collect();
                        let resp = ControlResponse::ok_with_data(
                            "Structural metrics for all 46 recipes",
                            serde_json::Value::Array(items),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!(
                            "{:<36} {:<34} {:>5} {:>8} {:>5} {:>5} {:>5} {:>7} {:>7} {:>8}",
                            "FIXTURE",
                            "RECIPE",
                            "NODES",
                            "EXPANDED",
                            "DEPTH",
                            "FILLS",
                            "GRADS",
                            "BORDERS",
                            "SHADOWS",
                            "TEXBYTES"
                        );
                        println!("{}", "-".repeat(130));
                        for (f, r, m) in &rows {
                            println!(
                                "{:<36} {:<34} {:>5} {:>8} {:>5} {:>5} {:>5} {:>7} {:>7} {:>8}",
                                f,
                                r,
                                m.graph_nodes,
                                m.expanded_ops,
                                m.max_depth,
                                m.fill_ops,
                                m.gradient_ops,
                                m.border_ops,
                                m.shadow_lobes,
                                m.texture_rgba_bytes
                            );
                        }
                        println!("{}", "-".repeat(130));
                        println!("Total: {} recipes", rows.len());
                    }
                    Ok(CliOutcome::Exit(0))
                }
            }
            CliRenderLabCommand::Recipes => {
                let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                let mut plans = Vec::new();
                for def in catalog {
                    if let Some(recipe) =
                        crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                    {
                        if let Some(plan) = crate::render_lab::plans::plan_for_recipe(recipe.id) {
                            plans.push((def.id.as_str(), plan));
                        }
                    }
                }

                if json {
                    let items: Vec<serde_json::Value> = plans
                        .iter()
                        .map(|(_, p)| serde_json::to_value(p).unwrap_or_default())
                        .collect();
                    let resp = ControlResponse::ok_with_data(
                        "All 46 canonical RecipePlans",
                        serde_json::Value::Array(items),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!("Canonical Recipe Plans ({} total):", plans.len());
                    for (fixture, plan) in &plans {
                        let m = plan.structural_metrics();
                        println!(
                            "  {:<36} -> {:<34} (nodes: {}, depth: {}, ops: {})",
                            fixture, plan.recipe_id, m.graph_nodes, m.max_depth, m.expanded_ops
                        );
                    }
                }
                Ok(CliOutcome::Exit(0))
            }
            CliRenderLabCommand::Recipe { id } => {
                let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                let (recipe_id, fixture_str) =
                    if let Some(def) = catalog.iter().find(|d| d.id.as_str() == id.as_str()) {
                        let r = crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                            .map(|rec| rec.id.to_string())
                            .unwrap_or_else(|| id.clone());
                        (r, Some(def.id.as_str().to_string()))
                    } else {
                        (id.clone(), None)
                    };

                let plan = match crate::render_lab::plans::plan_for_recipe(&recipe_id) {
                    Some(p) => p,
                    None => {
                        let resp =
                            ControlResponse::error(format!("Recipe plan not found for '{id}'"));
                        return outcome_from_response(&resp, json);
                    }
                };

                if json {
                    let resp = ControlResponse::ok_with_data(
                        format!("RecipePlan for {recipe_id}"),
                        serde_json::to_value(&plan).unwrap_or_default(),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!("Recipe Plan: {}", plan.recipe_id);
                    if let Some(f) = fixture_str {
                        println!("Fixture:     {}", f);
                    }
                    println!("Schema:      v{}", plan.schema_version);
                    let m = plan.structural_metrics();
                    println!(
                        "Metrics:     nodes={}, expanded={}, depth={}, fills={}, grads={}, borders={}, shadows={}, textures={} ({} bytes)",
                        m.graph_nodes, m.expanded_ops, m.max_depth, m.fill_ops, m.gradient_ops, m.border_ops, m.shadow_lobes, m.texture_count, m.texture_rgba_bytes
                    );
                    println!(
                        "Plan JSON:\n{}",
                        serde_json::to_string_pretty(&plan).unwrap_or_default()
                    );
                }
                Ok(CliOutcome::Exit(0))
            }
        },
        CliCommand::Appearance(appearance_cmd) => match appearance_cmd {
            CliAppearanceCommand::StyleGet => match ConfigManager::get_setting("visual-style") {
                Ok(val) => {
                    if json {
                        let resp = ControlResponse::ok_with_data(
                            format!("visual-style: {val}"),
                            serde_json::json!({ "key": "visual-style", "value": val }),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!("visual-style = {val}");
                    }
                    Ok(CliOutcome::Exit(0))
                }
                Err(e) => {
                    let resp = ControlResponse::error(e.to_string());
                    outcome_from_response(&resp, json)
                }
            },
            CliAppearanceCommand::StyleSet { value } => {
                if is_running {
                    let resp = ControlSocket::send_command(ControlCommand::SettingsSet {
                        key: "visual-style".to_string(),
                        value: value.clone(),
                    })
                    .await?;
                    outcome_from_response(&resp, json)
                } else {
                    match ConfigManager::set_setting("visual-style", &value) {
                        Ok(()) => {
                            let resp = ControlResponse::ok(format!(
                                "Visual style set to '{value}' (offline)"
                            ));
                            outcome_from_response(&resp, json)
                        }
                        Err(e) => {
                            let resp = ControlResponse::error(e.to_string());
                            outcome_from_response(&resp, json)
                        }
                    }
                }
            }
            CliAppearanceCommand::Status => {
                let config = ConfigManager::load_gpui_config();
                let status = crate::visual_style::resolver::status_for_config(
                    config.visual_style,
                    config.dark_theme,
                );
                if json {
                    let resp = ControlResponse::ok_with_data(
                        "Appearance status",
                        serde_json::to_value(&status).unwrap_or_default(),
                    );
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&resp).unwrap_or_default()
                    );
                } else {
                    println!("Appearance Status:");
                    println!("  Visual Style:   {}", status.visual_style);
                    println!("  Color Scheme:   {}", status.color_scheme);
                    println!("  Profile Rev:    {}", status.active_profile_revision);
                    println!("  Source:         {}", status.source);
                    println!("  Effects:");
                    for effect in &status.effects {
                        println!(
                            "    {}: requested, resolved {} — {}",
                            effect.kind, effect.resolved, effect.note
                        );
                    }
                }
                Ok(CliOutcome::Exit(0))
            }
            CliAppearanceCommand::Resolve { role } => match role {
                None => {
                    if json {
                        let roles: Vec<serde_json::Value> = crate::visual_style::SurfaceRole::ALL
                            .iter()
                            .map(|r| {
                                serde_json::json!({
                                    "id": r.as_str(),
                                    "label": r.label(),
                                    "treatment": r.treatment().as_str(),
                                    "requires_content_protection":
                                        r.requires_content_protection(),
                                })
                            })
                            .collect();
                        let resp = ControlResponse::ok_with_data(
                            "Surface roles",
                            serde_json::json!({ "roles": roles }),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!("Canonical surface roles:");
                        for r in crate::visual_style::SurfaceRole::ALL {
                            println!(
                                "  {:<18} {}  (treatment: {}, protected: {})",
                                r.as_str(),
                                r.label(),
                                r.treatment(),
                                r.requires_content_protection()
                            );
                        }
                    }
                    Ok(CliOutcome::Exit(0))
                }
                Some(role_str) => {
                    let role_parsed = crate::visual_style::SurfaceRole::parse(&role_str)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    let config = ConfigManager::load_gpui_config();
                    let projection = crate::visual_style::resolver::resolve_style(
                        config.visual_style,
                        role_parsed,
                        crate::visual_style::ColorScheme::from_dark_theme(config.dark_theme),
                    );
                    if json {
                        let resp = ControlResponse::ok_with_data(
                            "Style projection",
                            serde_json::to_value(&projection).unwrap_or_default(),
                        );
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&resp).unwrap_or_default()
                        );
                    } else {
                        println!("Style Projection:");
                        println!("  Style:          {}", projection.style);
                        println!(
                            "  Role:           {} ({})",
                            projection.role,
                            role_parsed.label()
                        );
                        println!("  Treatment:      {}", projection.treatment.label());
                        println!("  Color Scheme:   {}", projection.color_scheme);
                        println!(
                            "  Opaque:         {}",
                            if projection.opaque { "yes" } else { "no" }
                        );
                        println!(
                            "  Content Scrim:  {}",
                            if projection.content_scrim {
                                "yes"
                            } else {
                                "no"
                            }
                        );
                        println!("  Effects:");
                        for effect in &projection.effects {
                            println!(
                                "    {}: requested, resolved {} — {}",
                                effect.kind, effect.resolved, effect.note
                            );
                        }
                    }
                    Ok(CliOutcome::Exit(0))
                }
            },
        },
    }
}

pub(crate) fn outcome_from_response(resp: &ControlResponse, json: bool) -> Result<CliOutcome> {
    print_response(resp, json);
    if resp.ok {
        Ok(CliOutcome::Exit(0))
    } else {
        Ok(CliOutcome::Exit(1))
    }
}

fn print_response(resp: &ControlResponse, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(resp).unwrap_or_default());
    } else if resp.ok {
        if let Some(ref msg) = resp.message {
            println!("{msg}");
        }
    } else if let Some(ref err) = resp.error {
        eprintln!("Error: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_parse_empty_args_launches_gui() {
        let args = vec!["calix-gpui".to_string()];
        let parsed = CliInvocation::parse_from_args(&args).expect("Should parse empty args");
        assert!(!parsed.json);
        assert_eq!(parsed.command, None);
    }

    #[test]
    fn test_parse_json_flag_anywhere() {
        let args1 = vec![
            "calix-gpui".to_string(),
            "--json".to_string(),
            "status".to_string(),
        ];
        let parsed1 = CliInvocation::parse_from_args(&args1).expect("Should parse");
        assert!(parsed1.json);
        assert_eq!(parsed1.command, Some(CliCommand::Status));

        let args2 = vec![
            "calix-gpui".to_string(),
            "status".to_string(),
            "-j".to_string(),
        ];
        let parsed2 = CliInvocation::parse_from_args(&args2).expect("Should parse");
        assert!(parsed2.json);
        assert_eq!(parsed2.command, Some(CliCommand::Status));
    }

    #[test]
    fn test_parse_subcommands() {
        let cases = vec![
            (vec!["calix-gpui", "open"], Some(CliCommand::Open)),
            (vec!["calix-gpui", "focus"], Some(CliCommand::Focus)),
            (vec!["calix-gpui", "status"], Some(CliCommand::Status)),
            (vec!["calix-gpui", "quit"], Some(CliCommand::Quit)),
            (
                vec!["calix-gpui", "navigate", "installed"],
                Some(CliCommand::Navigate {
                    destination: "installed".to_string(),
                }),
            ),
            (
                vec!["calix-gpui", "search", "ripgrep"],
                Some(CliCommand::Search {
                    query: "ripgrep".to_string(),
                }),
            ),
            (
                vec!["calix-gpui", "view", "table"],
                Some(CliCommand::View {
                    mode: "table".to_string(),
                }),
            ),
            (
                vec!["calix-gpui", "inspect", "aalib"],
                Some(CliCommand::Inspect {
                    package: "aalib".to_string(),
                }),
            ),
            (
                vec!["calix-gpui", "inspector", "overview"],
                Some(CliCommand::Inspector {
                    tab: "overview".to_string(),
                }),
            ),
            (
                vec!["calix-gpui", "logs", "clear"],
                Some(CliCommand::Logs {
                    operation: "clear".to_string(),
                }),
            ),
            (
                vec!["calix-gpui", "settings", "list"],
                Some(CliCommand::Settings(CliSettingsCommand::List)),
            ),
            (
                vec!["calix-gpui", "settings", "get", "theme"],
                Some(CliCommand::Settings(CliSettingsCommand::Get {
                    key: "theme".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "settings", "set", "theme", "dark"],
                Some(CliCommand::Settings(CliSettingsCommand::Set {
                    key: "theme".to_string(),
                    value: "dark".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "settings", "reset", "theme"],
                Some(CliCommand::Settings(CliSettingsCommand::Reset {
                    key: Some("theme".to_string()),
                })),
            ),
            (
                vec!["calix-gpui", "settings", "reset", "all"],
                Some(CliCommand::Settings(CliSettingsCommand::Reset {
                    key: None,
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "open"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Open)),
            ),
            (
                vec![
                    "calix-gpui",
                    "render-lab",
                    "fixture",
                    "field.signed-voltage",
                ],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Fixture {
                    id: "field.signed-voltage".to_string(),
                })),
            ),
            (
                vec![
                    "calix-gpui",
                    "render-lab",
                    "material",
                    "painted-metal.signal-orange",
                ],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Material {
                    id: "painted-metal.signal-orange".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "topology", "floating-island"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Topology {
                    variant: "floating-island".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "motion", "smooth"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Motion {
                    variant: "smooth".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "quality", "stock"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Quality {
                    level: "stock".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "time", "0.500"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Time {
                    seconds: 0.5,
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "status"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Status)),
            ),
            (
                vec!["calix-gpui", "render-lab", "style", "transparency"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Style {
                    style: "transparency".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "ledger"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Ledger)),
            ),
            (
                vec!["calix-gpui", "render-lab", "metrics"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Metrics {
                    fixture: None,
                })),
            ),
            (
                vec![
                    "calix-gpui",
                    "render-lab",
                    "metrics",
                    "field.zero-positive-scalar",
                ],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Metrics {
                    fixture: Some("field.zero-positive-scalar".to_string()),
                })),
            ),
            (
                vec!["calix-gpui", "render-lab", "recipes"],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Recipes)),
            ),
            (
                vec![
                    "calix-gpui",
                    "render-lab",
                    "recipe",
                    "g01-linear-scalar-gradient/r1",
                ],
                Some(CliCommand::RenderLab(CliRenderLabCommand::Recipe {
                    id: "g01-linear-scalar-gradient/r1".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "appearance", "style", "get"],
                Some(CliCommand::Appearance(CliAppearanceCommand::StyleGet)),
            ),
            (
                vec!["calix-gpui", "appearance", "style", "set", "transparency"],
                Some(CliCommand::Appearance(CliAppearanceCommand::StyleSet {
                    value: "transparency".to_string(),
                })),
            ),
            (
                vec!["calix-gpui", "appearance", "status"],
                Some(CliCommand::Appearance(CliAppearanceCommand::Status)),
            ),
            (
                vec!["calix-gpui", "appearance", "resolve"],
                Some(CliCommand::Appearance(CliAppearanceCommand::Resolve {
                    role: None,
                })),
            ),
            (
                vec!["calix-gpui", "appearance", "resolve", "menu"],
                Some(CliCommand::Appearance(CliAppearanceCommand::Resolve {
                    role: Some("menu".to_string()),
                })),
            ),
        ];

        for (input, expected) in cases {
            let str_vec: Vec<String> = input.into_iter().map(|s| s.to_string()).collect();
            let parsed = CliInvocation::parse_from_args(&str_vec).expect("Parse error");
            assert_eq!(parsed.command, expected);
        }
    }

    #[test]
    fn test_parse_validation_failures() {
        let bad_view = vec![
            "calix-gpui".to_string(),
            "view".to_string(),
            "banana".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&bad_view).is_err());

        let missing_nav = vec!["calix-gpui".to_string(), "navigate".to_string()];
        assert!(CliInvocation::parse_from_args(&missing_nav).is_err());

        let missing_search = vec!["calix-gpui".to_string(), "search".to_string()];
        assert!(CliInvocation::parse_from_args(&missing_search).is_err());

        let missing_render_lab_sub = vec!["calix-gpui".to_string(), "render-lab".to_string()];
        assert!(CliInvocation::parse_from_args(&missing_render_lab_sub).is_err());

        let invalid_render_lab_sub = vec![
            "calix-gpui".to_string(),
            "render-lab".to_string(),
            "unknown".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&invalid_render_lab_sub).is_err());

        let invalid_time = vec![
            "calix-gpui".to_string(),
            "render-lab".to_string(),
            "time".to_string(),
            "abc".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&invalid_time).is_err());

        let negative_time = vec![
            "calix-gpui".to_string(),
            "render-lab".to_string(),
            "time".to_string(),
            "-1.0".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&negative_time).is_err());

        let invalid_lab_style = vec![
            "calix-gpui".to_string(),
            "render-lab".to_string(),
            "style".to_string(),
            "neon".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&invalid_lab_style).is_err());

        let missing_lab_style = vec![
            "calix-gpui".to_string(),
            "render-lab".to_string(),
            "style".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&missing_lab_style).is_err());

        let missing_recipe_id = vec![
            "calix-gpui".to_string(),
            "render-lab".to_string(),
            "recipe".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&missing_recipe_id).is_err());

        let missing_appearance_sub = vec!["calix-gpui".to_string(), "appearance".to_string()];
        assert!(CliInvocation::parse_from_args(&missing_appearance_sub).is_err());

        let invalid_appearance_sub = vec![
            "calix-gpui".to_string(),
            "appearance".to_string(),
            "theme".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&invalid_appearance_sub).is_err());

        let invalid_style_value = vec![
            "calix-gpui".to_string(),
            "appearance".to_string(),
            "style".to_string(),
            "set".to_string(),
            "neon".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&invalid_style_value).is_err());

        let missing_style_value = vec![
            "calix-gpui".to_string(),
            "appearance".to_string(),
            "style".to_string(),
            "set".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&missing_style_value).is_err());

        let invalid_role = vec![
            "calix-gpui".to_string(),
            "appearance".to_string(),
            "resolve".to_string(),
            "glass-card".to_string(),
        ];
        assert!(CliInvocation::parse_from_args(&invalid_role).is_err());
    }

    #[test]
    fn test_parse_internal_gui_flag() {
        let args = vec!["calix-gpui".to_string(), "--internal-gui".to_string()];
        let parsed = CliInvocation::parse_from_args(&args).expect("Should parse --internal-gui");
        assert!(parsed.internal_gui);
        assert_eq!(parsed.command, None);
        assert!(!parsed.json);

        let args_with_extra = vec![
            "calix-gpui".to_string(),
            "--internal-gui".to_string(),
            "something".to_string(),
        ];
        let parsed_extra =
            CliInvocation::parse_from_args(&args_with_extra).expect("Should parse --internal-gui");
        assert!(parsed_extra.internal_gui);
    }

    #[test]
    fn test_outcome_from_response_exit_codes() {
        let ok_resp = ControlResponse::ok("operation succeeded");
        let ok_outcome = outcome_from_response(&ok_resp, false).expect("outcome ok");
        match ok_outcome {
            CliOutcome::Exit(code) => assert_eq!(code, 0),
            _ => panic!("Expected CliOutcome::Exit(0)"),
        }

        let err_resp = ControlResponse::error("operation failed");
        let err_outcome = outcome_from_response(&err_resp, false).expect("outcome err");
        match err_outcome {
            CliOutcome::Exit(code) => assert_eq!(code, 1),
            _ => panic!("Expected CliOutcome::Exit(1)"),
        }
    }
}
