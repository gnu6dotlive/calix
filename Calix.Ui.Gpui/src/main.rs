#![deny(dead_code)]
#![deny(unused_variables)]
#![deny(unused_imports)]
#![deny(unused_must_use)]
#![recursion_limit = "8192"]

mod backend;
mod components;
mod config;
mod control;
pub mod brand;
pub mod icons;
pub mod render_lab;
mod state;
mod theme;
pub mod ui_metrics;
mod views;
pub mod visual_style;

use crate::components::search_input::*;
use gpui::*;
use views::WorkspaceView;

fn main() {
    env_logger::init();

    // Initialise et entre dans le contexte du runtime Tokio pour les tâches et processus async
    let _rt_guard = backend::process::runtime().enter();

    let args: Vec<String> = std::env::args().collect();
    let invocation = match crate::control::cli::CliInvocation::parse_from_args(&args) {
        Ok(inv) => inv,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    let is_json = invocation.json;
    let outcome =
        backend::process::runtime().block_on(crate::control::cli::run_cli_invocation(invocation));
    match outcome {
        Ok(crate::control::cli::CliOutcome::Exit(code)) => {
            std::process::exit(code);
        }
        Ok(crate::control::cli::CliOutcome::LaunchGui) => {}
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    // Single-instance lifetime lock (eliminates TOCTOU startup races)
    let _instance_lock = match crate::control::socket::InstanceLock::try_acquire() {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            // Another instance holds the lifetime lock.
            // Forward Open command to the primary instance over the control socket and propagate outcome exit code.
            let rt = backend::process::runtime();
            let forwarded = rt.block_on(async {
                for _ in 0..20 {
                    if let Ok(resp) = crate::control::socket::ControlSocket::send_command(
                        crate::control::protocol::ControlCommand::Open,
                    )
                    .await
                    {
                        return Ok(resp);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
                Err(anyhow::anyhow!("Timed out connecting to primary instance"))
            });
            match forwarded {
                Ok(resp) => {
                    let outcome = crate::control::cli::outcome_from_response(&resp, is_json);
                    match outcome {
                        Ok(crate::control::cli::CliOutcome::Exit(code)) => std::process::exit(code),
                        _ => std::process::exit(0),
                    }
                }
                Err(err) => {
                    eprintln!("Error forwarding to primary instance: {err}");
                    std::process::exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to acquire instance lock: {err}");
            std::process::exit(1);
        }
    };

    log::info!("Starting Calix GPUI desktop session...");

    let calix_settings = crate::config::ConfigManager::load_calix_settings();
    let gpui_config = crate::config::ConfigManager::load_gpui_config_sanitized();
    let initial_calix_settings = calix_settings.clone();
    let initial_gpui_config = gpui_config.clone();

    Application::with_platform(gpui_linux::current_platform(false))
        .with_assets(crate::icons::AppIcons::new())
        .run(move |cx: &mut App| {
            cx.bind_keys([
                KeyBinding::new("backspace", Backspace, Some("SearchInput")),
                KeyBinding::new("delete", Delete, Some("SearchInput")),
                KeyBinding::new("left", Left, Some("SearchInput")),
                KeyBinding::new("right", Right, Some("SearchInput")),
                KeyBinding::new("shift-left", SelectLeft, Some("SearchInput")),
                KeyBinding::new("shift-right", SelectRight, Some("SearchInput")),
                KeyBinding::new("ctrl-a", SelectAll, Some("SearchInput")),
                KeyBinding::new("cmd-a", SelectAll, Some("SearchInput")),
                KeyBinding::new("home", Home, Some("SearchInput")),
                KeyBinding::new("end", End, Some("SearchInput")),
                KeyBinding::new("ctrl-v", Paste, Some("SearchInput")),
                KeyBinding::new("cmd-v", Paste, Some("SearchInput")),
                KeyBinding::new("ctrl-c", Copy, Some("SearchInput")),
                KeyBinding::new("cmd-c", Copy, Some("SearchInput")),
                KeyBinding::new("ctrl-x", Cut, Some("SearchInput")),
                KeyBinding::new("cmd-x", Cut, Some("SearchInput")),
                KeyBinding::new("escape", Escape, Some("SearchInput")),
                KeyBinding::new("enter", Enter, Some("SearchInput")),
            ]);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(gpui_config.window_width), px(gpui_config.window_height)),
                })),
                window_min_size: Some(size(
                    px(crate::config::MIN_WINDOW_WIDTH),
                    px(crate::config::MIN_WINDOW_HEIGHT),
                )),
                titlebar: Some(TitlebarOptions {
                    title: Some("Calix — Universal Package Manager".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                app_id: Some("live.gnu6.calix".into()),
                ..Default::default()
            };

            let _ = cx.open_window(options, move |_, cx| {
                cx.new(|cx| {
                    WorkspaceView::with_config(initial_calix_settings, initial_gpui_config, cx)
                })
            });
        });

    crate::control::socket::ControlSocket::cleanup();
}
