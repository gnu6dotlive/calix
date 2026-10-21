use crate::backend::client::CalixClient;
use crate::backend::models::{ArchNewsItem, UnifiedPackage};
use crate::backend::process::LogStreamEvent;
use crate::components::toast_overlay::{ToastOverlay, ToastOverlayProps};
use crate::config::{ConfigManager, GpuiUiConfig, CalixSettings};
use crate::control::ipc::{ControlIpcMessage, ControlIpcServer};
use crate::control::protocol::{
    ControlCommand, ControlFilters, ControlResponse, ControlStatus, CONTROL_PROTOCOL_VERSION,
};
use crate::components::context_header::{ContextHeader, ContextHeaderProps};
use crate::state::package_store::SourceHealth;
use crate::state::query::SortMode;
use crate::state::{
    AppSession, ConsoleEvent, ConsoleModel, InspectorTab, MotionDurations, NavScope, Place,
    PackageKey, PackageSourceKind, PackageStore, PackageStoreEvent, PackageViewMode, SessionEvent,
    ToastAction, ToastCenter, ToastKind,
};
use crate::theme::Theme;
use crate::views::news::{NewsView, NewsViewProps};
use crate::views::operation_console::OperationConsoleView;
use crate::views::package_workstation::{PackageWorkstationConfig, PackageWorkstationView};
use crate::views::settings::{SettingsView, SettingsViewProps};
use crate::views::sidebar::SidebarView;
use crate::visual_style::VisualStyleId;
use gpui::ScrollStrategy;
use gpui::*;
use std::rc::Rc;
use tokio::sync::mpsc;

enum MutationAction {
    Install { is_aur: bool, is_flatpak: bool },
    Remove { is_flatpak: bool },
    UpgradeSystem,
}

pub struct WorkspaceView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
    pub console: Entity<ConsoleModel>,
    pub toast_center: Entity<ToastCenter>,
    pub workstation: Entity<PackageWorkstationView>,
    pub sidebar: Entity<SidebarView>,
    pub console_view: Entity<OperationConsoleView>,
    pub settings: SettingsView,
    pub render_lab: Entity<crate::render_lab::RenderLabState>,
    pub render_lab_ledger_collapsed: bool,
    pub render_lab_catalog_scroll: ScrollHandle,
    pub calix_settings: CalixSettings,
    pub gpui_config: GpuiUiConfig,
    pub theme: Theme,
    pub search_focus: FocusHandle,
    pub search_input_buffer: String,
    pub search_debounce_task: Option<Task<()>>,
    pub news: Vec<ArchNewsItem>,
    pub is_loading_news: bool,
    pub motion_policy: crate::state::motion::MotionPolicy,
}

impl WorkspaceView {
    pub fn with_config(
        calix_settings: CalixSettings,
        gpui_config: GpuiUiConfig,
        cx: &mut Context<Self>,
    ) -> Self {
        let theme = if gpui_config.dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        }
        .with_brand(gpui_config.brand_color);

        let client = CalixClient::new(None);
        let session = cx.new(|_cx| {
            let mut s = AppSession::with_initial_scope(gpui_config.last_scope);
            if gpui_config.compact_view {
                s.sidebar_collapsed = true;
            }
            s.view_mode = gpui_config.view_mode;
            s
        });
        let store = cx.new(|_cx| PackageStore::new(client));
        let console = cx.new(|_cx| {
            let mut c = ConsoleModel::new();
            c.is_open = gpui_config.log_drawer_open;
            c.auto_open = gpui_config.log_drawer_open;
            c
        });
        let toast_center = cx.new(|_cx| ToastCenter::new());

        let reduce_motion = gpui_config.reduce_motion;
        let compact = gpui_config.compact_view;
        let aur_enabled = calix_settings.aur_enabled;
        let flatpak_enabled = calix_settings.flat_pack_enabled;
        let appimage_enabled = calix_settings.app_image_enabled;

        session.update(cx, |s, cx| {
            s.clamp_source_scope(aur_enabled, flatpak_enabled, appimage_enabled, cx);
        });

        let workstation = cx.new(|cx| {
            PackageWorkstationView::new(
                session.clone(),
                store.clone(),
                console.clone(),
                toast_center.clone(),
                PackageWorkstationConfig {
                    theme,
                    reduce_motion,
                    compact,
                    aur_enabled,
                    flatpak_enabled,
                    appimage_enabled,
                    visual_style: gpui_config.visual_style,
                },
                cx,
            )
        });

        let sidebar = cx.new(|cx| {
            SidebarView::new(
                session.clone(),
                store.clone(),
                theme,
                reduce_motion,
                gpui_config.visual_style,
                cx,
            )
        });

        let console_view = cx.new(|cx| {
            OperationConsoleView::new(
                console.clone(),
                toast_center.clone(),
                gpui_config.log_drawer_height,
                theme,
                reduce_motion,
                cx,
            )
        });

        let settings = SettingsView::new(calix_settings.clone(), gpui_config.clone());

        // Abonnements réactifs aux événements de session
        cx.subscribe(&session, |this, _emitter, event, cx| match event {
            SessionEvent::PlaceChanged(place) => {
                log::debug!("Navigation vers le lieu : {:?}", place);
                this.on_place_changed(*place, cx);
                cx.notify();
            }
            SessionEvent::ScopeChanged(scope) => {
                log::debug!("Navigation vers le périmètre : {:?}", scope);
                this.on_scope_changed(*scope, cx);
                cx.notify();
            }
            SessionEvent::SourceScopeChanged(scope) => {
                log::debug!("Changement de scope de source : {:?}", scope);
                let query = this.session.read(cx).search_query.clone();
                if !query.trim().is_empty() {
                    this.execute_search(query, cx);
                }
                cx.notify();
            }
            SessionEvent::StateFilterChanged(_) | SessionEvent::SortModeChanged(_) => {
                cx.notify();
            }
            SessionEvent::SearchQueryChanged(q) => {
                log::debug!("Requête de recherche mise à jour : '{}'", q);
                this.workstation.update(cx, |ws, cx| {
                    ws.search_input.update(cx, |si, cx| {
                        if si.text() != q {
                            si.set_text(q.clone(), cx);
                        }
                    });
                });
                this.on_search_input(q.clone(), cx);
                cx.notify();
            }
            SessionEvent::SearchingStateChanged(searching) => {
                let searching_val = *searching;
                this.workstation.update(cx, |ws, cx| {
                    ws.search_input.update(cx, |si, cx| {
                        si.set_is_searching(searching_val, cx);
                    });
                });
                cx.notify();
            }
            SessionEvent::PackageSelected(opt_key) => {
                log::debug!("Paquet sélectionné : {:?}", opt_key);
                if let Some(key) = opt_key {
                    this.ensure_package_details(key.clone(), cx);
                    if this.session.read(cx).inspector_tab == InspectorTab::FilesBuild
                        && key.source == PackageSourceKind::Aur
                    {
                        this.ensure_pkgbuild(key.name.clone(), cx);
                    }
                }
                cx.notify();
            }
            SessionEvent::SidebarToggled(_) => {
                cx.notify();
            }
            SessionEvent::ViewModeChanged(mode) => {
                let mut config = crate::config::ConfigManager::load_gpui_config();
                config.view_mode = *mode;
                let _ = crate::config::ConfigManager::save_gpui_config(&config);
                cx.notify();
            }
            SessionEvent::InspectorTabChanged(tab) => {
                if *tab == InspectorTab::FilesBuild {
                    if let Some(ref key) = this.session.read(cx).selected_package_key {
                        if key.source == PackageSourceKind::Aur {
                            this.ensure_pkgbuild(key.name.clone(), cx);
                        }
                    }
                }
                cx.notify();
            }
        })
        .detach();

        // Abonnement réactif à SearchInput de PackageWorkstationView
        let search_input_ent = workstation.read(cx).search_input.clone();
        cx.subscribe(&search_input_ent, |this, _emitter, event, cx| match event {
            crate::components::search_input::SearchEvent::Changed(query) => {
                this.on_search_input(query.clone(), cx);
            }
            crate::components::search_input::SearchEvent::Submitted(query) => {
                this.execute_search(query.clone(), cx);
            }
            crate::components::search_input::SearchEvent::Cleared => {
                this.on_search_input(String::new(), cx);
            }
        })
        .detach();

        // Enregistrement du canal de notification pour la résolution d'identités
        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<PackageKey>();
        crate::components::package_identity::PackageIdentity::register_notifier(notify_tx);

        cx.spawn(async move |this, cx| {
            while let Some(key) = notify_rx.recv().await {
                let mut keys = vec![key];
                while let Ok(next) = notify_rx.try_recv() {
                    keys.push(next);
                }
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |_st, cx| {
                        for k in keys {
                            cx.emit(PackageStoreEvent::IdentityResolved(k));
                        }
                    });
                });
            }
        })
        .detach();

        // Abonnements réactifs typés aux événements du magasin de paquets et de la console
        cx.subscribe(&store, |this, _emitter, event, cx| match event {
            PackageStoreEvent::ResultsChanged => {
                let results = this.store.read(cx).active_results.clone();
                crate::components::package_identity::PackageIdentity::preload(&results);
                cx.notify();
            }
            PackageStoreEvent::PackageInvalidated(key) => {
                if this.session.read(cx).selected_package_key.as_ref() == Some(key) {
                    this.ensure_package_details(key.clone(), cx);
                }
                cx.notify();
            }
            PackageStoreEvent::UpdatesChanged(_) => {
                cx.notify();
            }
            PackageStoreEvent::InstalledChanged(_) => {
                cx.notify();
            }
            PackageStoreEvent::IdentityResolved(_) => {
                cx.notify();
            }
        })
        .detach();

        cx.subscribe(&console, |this, _emitter, event, cx| {
            if let ConsoleEvent::OperationFinished(status) = event {
                let reduce = this.gpui_config.reduce_motion;
                let (kind, title, msg, action) = match status {
                    crate::state::OperationStatus::Success(m) => {
                        (ToastKind::Success, "Operation successful", m.clone(), None)
                    }
                    crate::state::OperationStatus::Error(m) => (
                        ToastKind::Error,
                        "Operation failed",
                        m.clone(),
                        Some(ToastAction::OpenLogs),
                    ),
                    _ => return,
                };
                this.toast_center.update(cx, |tc, cx| {
                    tc.post(kind, title, msg, action, reduce, cx);
                });
                cx.notify();
            }
        })
        .detach();

        let render_lab = cx.new(|_| crate::render_lab::RenderLabState::new());

        let view = Self {
            session,
            store,
            console,
            toast_center,
            workstation,
            sidebar,
            console_view,
            settings,
            render_lab,
            render_lab_ledger_collapsed: false,
            render_lab_catalog_scroll: ScrollHandle::new(),
            calix_settings,
            gpui_config: gpui_config.clone(),
            theme,
            search_focus: cx.focus_handle(),
            search_input_buffer: String::new(),
            search_debounce_task: None,
            news: Vec::new(),
            is_loading_news: false,
            motion_policy: crate::state::motion::MotionPolicy::new(gpui_config.reduce_motion),
        };

        // Liaison des mutations depuis PackageWorkstationView vers WorkspaceView
        let entity_ws = cx.entity().clone();
        view.workstation.update(cx, |ws, _cx| {
            let entity_mut = entity_ws.clone();
            ws.set_on_mutation(Rc::new(move |pkg, is_install, _window, cx| {
                let pkg_clone = pkg.clone();
                let key = pkg.key();
                let is_aur = pkg.source_type == "AUR";
                let is_flatpak = pkg.source_type == "Flatpak";
                // The product's own command: the one it shows, above the log
                // that shows what the backend actually ran.
                let product_command = if is_install {
                    crate::state::canonical_install_command(pkg)
                } else {
                    crate::state::canonical_remove_command(pkg)
                };
                entity_mut.update(cx, |view, cx| {
                    view.console.update(cx, |c, cx| {
                        c.set_product_command(product_command.clone(), cx);
                    });
                    if is_install {
                        view.run_package_mutation(
                            &format!("Installation of {}", pkg_clone.name),
                            MutationAction::Install { is_aur, is_flatpak },
                            pkg_clone.name.clone(),
                            key,
                            cx,
                        );
                    } else {
                        view.run_package_mutation(
                            &format!("Uninstallation of {}", pkg_clone.name),
                            MutationAction::Remove { is_flatpak },
                            pkg_clone.name.clone(),
                            key,
                            cx,
                        );
                    }
                });
            }));
            let entity_upg = entity_ws.clone();
            ws.set_on_upgrade_all(Rc::new(move |_window, cx| {
                entity_upg.update(cx, |view, cx| {
                    view.run_package_mutation(
                        "Full system upgrade",
                        MutationAction::UpgradeSystem,
                        "system".to_string(),
                        PackageKey::new(PackageSourceKind::Alpm, "system-upgrade", None),
                        cx,
                    );
                });
            }));
            let entity_retry = entity_ws.clone();
            ws.on_retry_source = Some(Rc::new(move |kind, _w, cx| {
                entity_retry.update(cx, |view, cx| {
                    view.retry_source_search(kind, cx);
                });
            }));
            let entity_rel_inst = entity_ws.clone();
            ws.on_reload_installed = Some(Rc::new(move |_w, cx| {
                entity_rel_inst.update(cx, |view, cx| {
                    view.load_installed_packages(cx);
                });
            }));
            let entity_rel_upd = entity_ws.clone();
            ws.on_reload_updates = Some(Rc::new(move |_w, cx| {
                entity_rel_upd.update(cx, |view, cx| {
                    view.load_updates(cx);
                });
            }));
            let entity_rel_det = entity_ws.clone();
            ws.on_retry_details = Some(Rc::new(move |key, _w, cx| {
                entity_rel_det.update(cx, |view, cx| {
                    view.ensure_package_details(key.clone(), cx);
                });
            }));
        });

        // Enregistrement du canal IPC pour le contrôle frontend natif
        let (control_tx, mut control_rx) = tokio::sync::mpsc::channel::<ControlIpcMessage>(32);
        let _ = ControlIpcServer::start(control_tx);

        cx.spawn(async move |this, cx| {
            while let Some(msg) = control_rx.recv().await {
                let ControlIpcMessage { command, responder } = msg;
                let resp = this.update(cx, |view, cx| view.handle_control_command(command, cx));
                let final_resp = match resp {
                    Ok(r) => r,
                    Err(_) => ControlResponse::error("UI event loop is unavailable"),
                };
                let _ = responder.send(final_resp);
            }
        })
        .detach();

        // Chargement initial asynchrone non-bloquant
        view.trigger_initial_load(cx);

        view
    }

    pub fn handle_control_command(
        &mut self,
        cmd: ControlCommand,
        cx: &mut Context<Self>,
    ) -> ControlResponse {
        match cmd {
            ControlCommand::Open | ControlCommand::Focus => {
                cx.activate(true);
                if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
                    let _ = std::process::Command::new("hyprctl")
                        .args(["dispatch", "focuswindow", "class:calix-gpui"])
                        .output();
                }
                ControlResponse::ok("Calix window focused")
            }
            ControlCommand::Quit => {
                crate::control::socket::ControlSocket::cleanup();
                cx.quit();
                ControlResponse::ok("Calix GUI exiting")
            }
            ControlCommand::Status => {
                let session = self.session.read(cx);
                let console = self.console.read(cx);
                let status = ControlStatus {
                    protocol_version: CONTROL_PROTOCOL_VERSION,
                    gui_running: true,
                    pid: std::process::id(),
                    executable: std::env::current_exe()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| "/usr/lib/calix/calix-gpui".to_string()),
                    place: session.place.label().to_lowercase(),
                    scope: session.scope.as_str().to_string(),
                    scope_sources: session
                        .scope
                        .sources()
                        .iter()
                        .map(|source| source.to_string())
                        .collect(),
                    inventory_only_sources: session
                        .scope
                        .inventory_only_sources()
                        .iter()
                        .map(|source| source.to_string())
                        .collect(),
                    filters: ControlFilters {
                        sources: session
                            .source_scope
                            .sources_for(
                                session.scope,
                                self.calix_settings.aur_enabled,
                                self.calix_settings.flat_pack_enabled,
                                self.calix_settings.app_image_enabled,
                            )
                            .iter()
                            .map(|source| source.as_str().to_lowercase())
                            .collect(),
                        state: session.state_filter.label().to_lowercase(),
                        sort: session.sort_mode.label().to_lowercase(),
                        view: match session.view_mode {
                            PackageViewMode::Table => "table".to_string(),
                            PackageViewMode::Cards => "cards".to_string(),
                        },
                    },
                    row_count: self.current_display_packages(cx).len(),
                    checked_at: None,
                    destination: session.scope.as_str().to_string(),
                    query: session.search_query.clone(),
                    view_mode: match session.view_mode {
                        PackageViewMode::Table => "table".to_string(),
                        PackageViewMode::Cards => "cards".to_string(),
                    },
                    inspector_tab: match session.inspector_tab {
                        InspectorTab::Overview => "overview".to_string(),
                        InspectorTab::Dependencies => "dependencies".to_string(),
                        InspectorTab::FilesBuild => "files".to_string(),
                    },
                    selected_package: session
                        .selected_package_key
                        .as_ref()
                        .map(|k| k.name.clone()),
                    operation_running: matches!(
                        console.status,
                        crate::components::log_drawer::OperationStatus::Running(_)
                    ) || session.is_searching,
                    render_lab_active: session.render_lab,
                };
                ControlResponse::ok_with_data(
                    "Running",
                    serde_json::to_value(&status).unwrap_or_default(),
                )
            }
            ControlCommand::Navigate { destination } => {
                // The old verbs keep their meaning: the three package ones are
                // the scopes, the two others are the places.
                let target = match destination.to_ascii_lowercase().as_str() {
                    "browse" | "results" | "search" => Some((Place::Packages, Some(NavScope::Results))),
                    "installed" => Some((Place::Packages, Some(NavScope::Installed))),
                    "updates" => Some((Place::Packages, Some(NavScope::Updates))),
                    "news" => Some((Place::News, None)),
                    "settings" => Some((Place::Settings, None)),
                    _ => None,
                };
                match target {
                    Some((place, scope)) => {
                        self.session.update(cx, |s, cx| {
                            if let Some(scope) = scope {
                                s.set_scope(scope, cx);
                            }
                            s.set_place(place, cx);
                        });
                        ControlResponse::ok(format!("Navigated to {}", destination))
                    }
                    None => ControlResponse::error(format!(
                        "Invalid destination '{}'. Expected 'updates', 'installed', 'browse', 'news', or 'settings'",
                        destination
                    )),
                }
            }
            ControlCommand::Scope { scope } => {
                match NavScope::parse(&scope) {
                    Ok(scope) => {
                        self.session.update(cx, |s, cx| {
                            s.set_scope(scope, cx);
                            s.set_place(Place::Packages, cx);
                        });
                        ControlResponse::ok(format!("Scope set to {}", scope))
                    }
                    Err(message) => ControlResponse::error(message),
                }
            }
            ControlCommand::Search { query } => {
                self.session.update(cx, |s, cx| {
                    s.set_scope(NavScope::Results, cx);
                    s.set_place(Place::Packages, cx);
                    s.search_query = query.clone();
                });
                self.workstation.update(cx, |ws, cx| {
                    ws.search_input.update(cx, |si, cx| {
                        si.set_text(query.clone(), cx);
                    });
                });
                self.on_search_input(query.clone(), cx);
                ControlResponse::ok(format!("Searching for '{}'", query))
            }
            ControlCommand::View { mode } => match mode.to_ascii_lowercase().as_str() {
                "table" => {
                    let mut config = ConfigManager::load_gpui_config();
                    config.view_mode = PackageViewMode::Table;
                    if let Err(e) = ConfigManager::save_gpui_config(&config) {
                        return ControlResponse::error(e.to_string());
                    }
                    self.apply_committed_settings_to_runtime(cx);
                    ControlResponse::ok("View mode set to table")
                }
                "cards" => {
                    let mut config = ConfigManager::load_gpui_config();
                    config.view_mode = PackageViewMode::Cards;
                    if let Err(e) = ConfigManager::save_gpui_config(&config) {
                        return ControlResponse::error(e.to_string());
                    }
                    self.apply_committed_settings_to_runtime(cx);
                    ControlResponse::ok("View mode set to cards")
                }
                other => ControlResponse::error(format!(
                    "Invalid view mode '{}', expected 'table' or 'cards'",
                    other
                )),
            },
            ControlCommand::Inspect { package } => {
                let (req_source, req_name) =
                    if let Some((src_str, name_str)) = package.split_once(':') {
                        let src_kind = match src_str.to_ascii_lowercase().as_str() {
                            "alpm" => Some(PackageSourceKind::Alpm),
                            "aur" => Some(PackageSourceKind::Aur),
                            "flatpak" => Some(PackageSourceKind::Flatpak),
                            "appimage" => Some(PackageSourceKind::AppImage),
                            "cargo" => Some(PackageSourceKind::Cargo),
                            "npm" => Some(PackageSourceKind::Npm),
                            "release" => Some(PackageSourceKind::Release),
                            _ => None,
                        };
                        if let Some(src) = src_kind {
                            (Some(src), name_str)
                        } else {
                            (None, package.as_str())
                        }
                    } else {
                        (None, package.as_str())
                    };

                let store = self.store.read(cx);
                let mut candidates: Vec<PackageKey> = Vec::new();

                let mut check_pkg = |key: PackageKey, name: &str| {
                    if let Some(src) = req_source {
                        if key.source != src {
                            return;
                        }
                    }
                    if name.eq_ignore_ascii_case(req_name) && !candidates.contains(&key) {
                        candidates.push(key);
                    }
                };

                for p in store.active_results.iter() {
                    check_pkg(p.key(), &p.name);
                }
                for p in store.installed_packages.iter() {
                    check_pkg(p.key(), &p.name);
                }
                for p in store.updates_packages.iter() {
                    check_pkg(p.key(), &p.name);
                }
                for cached_list in store.search_cache.values() {
                    for p in cached_list.iter() {
                        check_pkg(p.key(), &p.name);
                    }
                }

                if candidates.is_empty() {
                    // Try authoritative standard ALPM backend query if req_source is None or Alpm
                    if req_source.is_none() || req_source == Some(PackageSourceKind::Alpm) {
                        let bin = store.client.binary_path.clone();
                        if let Ok(output) = std::process::Command::new(&bin)
                            .args(["search", "standard", "-d", req_name, "-j"])
                            .output()
                        {
                            if output.status.success() {
                                let text = String::from_utf8_lossy(&output.stdout);
                                if let Ok(Some(pkg)) = serde_json::from_str::<
                                    crate::backend::models::AlpmPackage,
                                >(&text)
                                .map(Some)
                                .or_else(|_| {
                                    serde_json::from_str::<
                                            Vec<crate::backend::models::AlpmPackage>,
                                        >(&text)
                                        .map(|v| v.into_iter().next())
                                }) {
                                    if pkg.name.eq_ignore_ascii_case(req_name) {
                                        candidates.push(PackageKey::new(
                                            PackageSourceKind::Alpm,
                                            &pkg.name,
                                            None,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }

                if candidates.is_empty() {
                    ControlResponse::error(format!(
                        "Package '{}' not found in active collections or system database",
                        package
                    ))
                } else if candidates.len() > 1 {
                    let sources: Vec<&str> = candidates.iter().map(|k| k.source.as_str()).collect();
                    ControlResponse::error(format!(
                        "Ambiguous package name '{}'. Found matches in multiple sources: [{}]. Please specify with source prefix (e.g. '{}:{}').",
                        package,
                        sources.join(", "),
                        sources[0],
                        req_name
                    ))
                } else {
                    let key = candidates.remove(0);
                    self.session.update(cx, |s, cx| {
                        s.select_package(Some(key.clone()), cx);
                    });
                    ControlResponse::ok(format!(
                        "Inspecting package '{}' ({})",
                        key.name,
                        key.source.as_str()
                    ))
                }
            }
            ControlCommand::Inspector { tab } => {
                let target_tab = match tab.to_ascii_lowercase().as_str() {
                    "overview" => Some(InspectorTab::Overview),
                    "dependencies" | "deps" => Some(InspectorTab::Dependencies),
                    "files" | "files-build" | "build" => Some(InspectorTab::FilesBuild),
                    _ => None,
                };
                if let Some(target) = target_tab {
                    self.session
                        .update(cx, |s, cx| s.set_inspector_tab(target, cx));
                    ControlResponse::ok(format!("Inspector tab set to '{}'", target.label()))
                } else {
                    ControlResponse::error(format!(
                        "Invalid inspector tab '{}', expected: overview, dependencies, files",
                        tab
                    ))
                }
            }
            ControlCommand::Logs { operation } => match operation.to_ascii_lowercase().as_str() {
                "show" | "open" => {
                    let mut config = ConfigManager::load_gpui_config();
                    config.log_drawer_open = true;
                    let _ = ConfigManager::save_gpui_config(&config);
                    self.apply_committed_settings_to_runtime(cx);
                    ControlResponse::ok("Logs drawer opened")
                }
                "hide" | "close" => {
                    let mut config = ConfigManager::load_gpui_config();
                    config.log_drawer_open = false;
                    let _ = ConfigManager::save_gpui_config(&config);
                    self.apply_committed_settings_to_runtime(cx);
                    ControlResponse::ok("Logs drawer closed")
                }
                "clear" => {
                    self.console.update(cx, |c, cx| {
                        c.clear_logs(cx);
                    });
                    ControlResponse::ok("Logs cleared")
                }
                other => ControlResponse::error(format!(
                    "Invalid logs operation '{}', expected: show, hide, clear",
                    other
                )),
            },
            ControlCommand::SettingsList => {
                let entries = ConfigManager::list_settings();
                ControlResponse::ok_with_data(
                    "Settings listed",
                    serde_json::to_value(&entries).unwrap_or_default(),
                )
            }
            ControlCommand::SettingsGet { key } => match ConfigManager::get_setting(&key) {
                Ok(val) => ControlResponse::ok_with_data(
                    format!("{key}: {val}"),
                    serde_json::json!({ "key": key, "value": val }),
                ),
                Err(e) => ControlResponse::error(e.to_string()),
            },
            ControlCommand::SettingsSet { key, value } => {
                if self.settings.is_dirty {
                    return ControlResponse::error(
                        "Settings edit conflict: settings view has uncommitted changes in GUI. Save or reset GUI draft first."
                    );
                }
                match ConfigManager::set_setting(&key, &value) {
                    Ok(()) => {
                        self.apply_committed_settings_to_runtime(cx);
                        ControlResponse::ok(format!("Setting '{key}' set to '{value}'"))
                    }
                    Err(e) => ControlResponse::error(e.to_string()),
                }
            }
            ControlCommand::SettingsReset { key } => {
                if self.settings.is_dirty {
                    return ControlResponse::error(
                        "Settings edit conflict: settings view has uncommitted changes in GUI. Save or reset GUI draft first."
                    );
                }
                let res = match key.as_deref() {
                    Some(k) => ConfigManager::reset_setting(k),
                    None => ConfigManager::reset_all(),
                };
                match res {
                    Ok(()) => {
                        self.apply_committed_settings_to_runtime(cx);
                        let target = key.as_deref().unwrap_or("all settings");
                        ControlResponse::ok(format!("Reset '{target}' to default"))
                    }
                    Err(e) => ControlResponse::error(e.to_string()),
                }
            }
            ControlCommand::RenderLabOpen => {
                self.session.update(cx, |s, cx| {
                    s.set_render_lab(true, cx);
                });
                ControlResponse::ok("Render Lab opened")
            }
            ControlCommand::RenderLabFixture { id } => {
                let res = self
                    .render_lab
                    .update(cx, |rl, cx| rl.set_fixture_by_id_str(&id, cx));
                match res {
                    Ok(()) => ControlResponse::ok(format!("Active fixture set to '{id}'")),
                    Err(e) => ControlResponse::error(e),
                }
            }
            ControlCommand::RenderLabTopology { variant } => {
                match crate::render_lab::TopologyVariant::parse(&variant) {
                    Ok(top) => {
                        self.render_lab
                            .update(cx, |rl, cx| rl.set_topology(top, cx));
                        ControlResponse::ok(format!("Topology variant set to '{}'", top.as_str()))
                    }
                    Err(e) => ControlResponse::error(e),
                }
            }
            ControlCommand::RenderLabMotion { variant } => {
                match crate::render_lab::MotionVariant::parse(&variant) {
                    Ok(mot) => {
                        self.render_lab.update(cx, |rl, cx| rl.set_motion(mot, cx));
                        ControlResponse::ok(format!("Motion variant set to '{}'", mot.as_str()))
                    }
                    Err(e) => ControlResponse::error(e),
                }
            }
            ControlCommand::RenderLabQuality { level } => {
                match crate::render_lab::QualityLevel::parse(&level) {
                    Ok(ql) => {
                        self.render_lab.update(cx, |rl, cx| rl.set_quality(ql, cx));
                        ControlResponse::ok(format!("Quality level set to '{}'", ql.as_str()))
                    }
                    Err(e) => ControlResponse::error(e),
                }
            }
            ControlCommand::RenderLabTime { seconds } => {
                if seconds < 0.0 || seconds.is_nan() || seconds.is_infinite() {
                    ControlResponse::error(format!(
                        "Invalid time '{seconds}', expected non-negative number"
                    ))
                } else {
                    self.render_lab.update(cx, |rl, cx| {
                        rl.set_clock(crate::render_lab::ClockMode::Frozen(seconds), cx);
                    });
                    ControlResponse::ok(format!("Clock frozen at t = {seconds:.3}s"))
                }
            }
            ControlCommand::RenderLabStatus => {
                let manifest = {
                    let rl = self.render_lab.read(cx);
                    crate::render_lab::RenderLabManifest::from_state(rl, true)
                };
                ControlResponse::ok_with_data(
                    "Render Lab Status",
                    serde_json::to_value(&manifest).unwrap_or_default(),
                )
            }
            ControlCommand::RenderLabStyle { style } => {
                match crate::visual_style::VisualStyleId::parse(&style) {
                    Ok(parsed) => {
                        self.render_lab
                            .update(cx, |rl, cx| rl.set_style(parsed, cx));
                        ControlResponse::ok(format!("Lab style axis set to '{}'", parsed.as_str()))
                    }
                    Err(e) => ControlResponse::error(e),
                }
            }
            ControlCommand::RenderLabMetrics { fixture } => {
                let target_fixture = match fixture {
                    Some(f) => Some(f),
                    None => {
                        let rl = self.render_lab.read(cx);
                        rl.active_fixture.as_ref().map(|f| f.as_str().to_string())
                    }
                };

                if let Some(fid) = target_fixture {
                    let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                    let fdef = catalog.iter().find(|d| d.id.as_str() == fid.as_str());
                    if let Some(def) = fdef {
                        if let Some(recipe) =
                            crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                        {
                            if let Some(plan) = crate::render_lab::plans::plan_for_recipe(recipe.id)
                            {
                                let metrics = plan.structural_metrics();
                                ControlResponse::ok_with_data(
                                    format!("Structural metrics for {}", recipe.id),
                                    serde_json::json!({
                                        "fixture": def.id.as_str(),
                                        "recipe": recipe.id,
                                        "metrics": metrics,
                                    }),
                                )
                            } else {
                                ControlResponse::error(format!(
                                    "No plan for recipe '{}'",
                                    recipe.id
                                ))
                            }
                        } else {
                            ControlResponse::error(format!(
                                "No recipe for fixture '{}'",
                                def.id.as_str()
                            ))
                        }
                    } else if let Some(plan) = crate::render_lab::plans::plan_for_recipe(&fid) {
                        let metrics = plan.structural_metrics();
                        ControlResponse::ok_with_data(
                            format!("Structural metrics for {}", plan.recipe_id),
                            serde_json::json!({
                                "recipe": plan.recipe_id,
                                "metrics": metrics,
                            }),
                        )
                    } else {
                        ControlResponse::error(format!("Unknown fixture or recipe '{fid}'"))
                    }
                } else {
                    let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                    let mut list = Vec::new();
                    for def in catalog {
                        if let Some(recipe) =
                            crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                        {
                            if let Some(plan) = crate::render_lab::plans::plan_for_recipe(recipe.id)
                            {
                                list.push(serde_json::json!({
                                    "fixture": def.id.as_str(),
                                    "recipe": recipe.id,
                                    "metrics": plan.structural_metrics(),
                                }));
                            }
                        }
                    }
                    ControlResponse::ok_with_data(
                        "Structural metrics for all 46 recipes",
                        serde_json::Value::Array(list),
                    )
                }
            }
            ControlCommand::RenderLabRecipes => {
                let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                let mut list = Vec::new();
                for def in catalog {
                    if let Some(recipe) =
                        crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                    {
                        if let Some(plan) = crate::render_lab::plans::plan_for_recipe(recipe.id) {
                            list.push(serde_json::to_value(&plan).unwrap_or_default());
                        }
                    }
                }
                ControlResponse::ok_with_data(
                    "All 46 canonical RecipePlans",
                    serde_json::Value::Array(list),
                )
            }
            ControlCommand::RenderLabRecipe { id } => {
                let catalog = crate::render_lab::catalog::FixtureCatalog::all();
                let recipe_id =
                    if let Some(def) = catalog.iter().find(|d| d.id.as_str() == id.as_str()) {
                        crate::render_lab::recipe::RecipeCatalog::find(def.id.as_str())
                            .map(|r| r.id.to_string())
                    } else {
                        Some(id.clone())
                    };

                if let Some(rid) = recipe_id {
                    if let Some(plan) = crate::render_lab::plans::plan_for_recipe(&rid) {
                        ControlResponse::ok_with_data(
                            format!("Recipe plan for {rid}"),
                            serde_json::to_value(&plan).unwrap_or_default(),
                        )
                    } else {
                        ControlResponse::error(format!("Recipe plan not found for '{id}'"))
                    }
                } else {
                    ControlResponse::error(format!("Recipe not found for '{id}'"))
                }
            }
        }
    }

    /// Single authority synchronization: loads persisted settings from disk and updates
    /// all runtime state, draft settings, and view components.
    pub fn apply_committed_settings_to_runtime(&mut self, cx: &mut Context<Self>) {
        let calix = ConfigManager::load_calix_settings();
        let gpui = ConfigManager::load_gpui_config_sanitized();

        let old_aur = self.calix_settings.aur_enabled;
        let old_flatpak = self.calix_settings.flat_pack_enabled;
        let old_appimage = self.calix_settings.app_image_enabled;

        self.calix_settings = calix.clone();
        self.gpui_config = gpui.clone();
        self.settings.reset_to(calix.clone(), gpui.clone());

        self.theme = if gpui.dark_theme {
            Theme::dark()
        } else {
            Theme::light()
        }
        .with_brand(gpui.brand_color);
        let theme = self.theme;

        self.motion_policy = crate::state::motion::MotionPolicy::new(gpui.reduce_motion);
        let reduce = self.motion_policy.reduce_motion;

        self.session.update(cx, |s, cx| {
            s.set_view_mode(gpui.view_mode, cx);
            s.set_sidebar_collapsed(gpui.compact_view, cx);
            s.clamp_source_scope(
                calix.aur_enabled,
                calix.flat_pack_enabled,
                calix.app_image_enabled,
                cx,
            );
        });

        self.workstation.update(cx, |ws, cx| {
            ws.set_theme(theme, cx);
            ws.set_visual_style(gpui.visual_style, cx);
            ws.set_reduce_motion(reduce, cx);
            ws.set_compact(gpui.compact_view, cx);
            ws.set_sources_enabled(
                calix.aur_enabled,
                calix.flat_pack_enabled,
                calix.app_image_enabled,
                cx,
            );
        });

        self.sidebar.update(cx, |sb, cx| {
            sb.set_theme(theme, cx);
            sb.set_visual_style(gpui.visual_style, cx);
            sb.set_reduce_motion(reduce, cx);
        });

        self.console_view.update(cx, |cv, cx| {
            cv.set_theme(theme, cx);
            cv.set_reduce_motion(reduce, cx);
            cv.set_configured_height(gpui.log_drawer_height, cx);
        });

        self.console.update(cx, |c, _| {
            c.set_auto_open(gpui.log_drawer_open);
            c.is_open = gpui.log_drawer_open;
        });

        let sources_changed = old_aur != calix.aur_enabled
            || old_flatpak != calix.flat_pack_enabled
            || old_appimage != calix.app_image_enabled;

        if sources_changed {
            self.store.update(cx, |st, _| {
                st.search_cache.clear();
            });
            let query = self.session.read(cx).search_query.clone();
            if !query.trim().is_empty() {
                self.execute_search(query, cx);
            }
        }

        cx.notify();
    }

    /// Déclenche le chargement initial en arrière-plan
    fn trigger_initial_load(&self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            let res = client.list_updates().await;
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(updates) => {
                        let unified: Vec<UnifiedPackage> = updates
                            .into_iter()
                            .map(UnifiedPackage::from_update)
                            .collect();
                        crate::components::package_identity::PackageIdentity::preload(&unified);
                        st.set_updates_packages(unified, cx);
                    }
                    Err(e) => {
                        log::warn!("Initial updates check failed: {:?}", e);
                        st.set_updates_error(e.to_string(), cx);
                    }
                });
            });
        })
        .detach();

        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            // The daemon query runs alongside the CLI one: the two answer
            // different halves of the same list, and the inventory takes
            // long enough that serialising them would be visible.
            let (res, snapshot) = tokio::join!(
                client.search_installed(""),
                crate::backend::calix_daemon::inventory()
            );
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(installed) => {
                        let unified = merged_installed(installed, &snapshot);
                        crate::components::package_identity::PackageIdentity::preload(&unified);
                        st.set_installed_packages(unified, cx);
                    }
                    Err(e) => {
                        log::warn!("Initial installed packages check failed: {:?}", e);
                        st.set_installed_error(e.to_string(), cx);
                    }
                });
            });
        })
        .detach();
    }

    /// Réaction au changement de lieu : la liste repart du haut.
    fn on_place_changed(&mut self, place: Place, cx: &mut Context<Self>) {
        self.workstation.update(cx, |ws, _cx| {
            ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
        });
        if place == Place::News {
            self.on_news_needed(cx);
        }
    }

    /// Réaction au changement de périmètre : le périmètre charge ses propres lignes.
    fn on_scope_changed(&mut self, scope: NavScope, cx: &mut Context<Self>) {
        self.workstation.update(cx, |ws, _cx| {
            ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
        });

        match scope {
            NavScope::Results => {
                let query = self.session.read(cx).search_query.clone();
                if !query.trim().is_empty() {
                    self.execute_search(query, cx);
                }
            }
            NavScope::Installed => {
                let is_empty = self.store.read(cx).installed_packages.is_empty();
                if is_empty {
                    self.load_installed_packages(cx);
                }
            }
            NavScope::Updates => {
                let is_empty = self.store.read(cx).updates_packages.is_empty();
                if is_empty {
                    self.load_updates(cx);
                }
            }
        }
    }

    /// Réaction au changement de lieu : Actualités charge son propre flux.
    fn on_news_needed(&mut self, cx: &mut Context<Self>) {
        if self.news.is_empty() {
            self.load_news(cx);
        }
    }

    /// Charge les paquets installés localement via le backend Calix,
    /// complétés par l'inventaire multi-écosystèmes maintained par le moteur Calix
    fn load_installed_packages(&mut self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            // The daemon query runs alongside the CLI one: the two answer
            // different halves of the same list, and the inventory takes
            // long enough that serialising them would be visible.
            let (res, snapshot) = tokio::join!(
                client.search_installed(""),
                crate::backend::calix_daemon::inventory()
            );
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(packages) => {
                        let unified = merged_installed(packages, &snapshot);
                        crate::components::package_identity::PackageIdentity::preload(&unified);
                        st.set_installed_packages(unified, cx);
                    }
                    Err(e) => {
                        log::error!("Failed to load installed packages: {:?}", e);
                        st.set_installed_error(e.to_string(), cx);
                    }
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Charge les mises à jour disponibles via le backend Calix
    fn load_updates(&mut self, cx: &mut Context<Self>) {
        let client = self.store.read(cx).client.clone();
        cx.spawn(async move |this, cx| {
            let res = client.list_updates().await;
            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(packages) => {
                        let unified: Vec<UnifiedPackage> = packages
                            .into_iter()
                            .map(UnifiedPackage::from_update)
                            .collect();
                        crate::components::package_identity::PackageIdentity::preload(&unified);
                        st.set_updates_packages(unified, cx);
                    }
                    Err(e) => {
                        log::error!("Failed to list updates: {:?}", e);
                        st.set_updates_error(e.to_string(), cx);
                    }
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Charge les actualités officielles Arch Linux
    fn load_news(&mut self, cx: &mut Context<Self>) {
        self.is_loading_news = true;
        let client = self.store.read(cx).client.clone();

        cx.spawn(async move |this, cx| {
            let res = client.list_news().await;
            let _ = this.update(cx, |view, cx| {
                view.is_loading_news = false;
                match res {
                    Ok(news_items) => {
                        view.news = news_items;
                        view.store.update(cx, |st, _cx| {
                            st.news_error = None;
                        });
                    }
                    Err(e) => {
                        log::error!("Failed to list news: {:?}", e);
                        view.news = Vec::new();
                        view.store.update(cx, |st, _cx| {
                            st.news_error = Some(e.to_string());
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// S'assure que la fiche détaillée du paquet est disponible dans le cache
    fn ensure_package_details(&mut self, key: PackageKey, cx: &mut Context<Self>) {
        if self.store.read(cx).get_cached_details(&key).is_some() {
            return;
        }

        if key.source == PackageSourceKind::Alpm {
            let client = self.store.read(cx).client.clone();
            let pkg_name = key.name.clone();
            let key_clone = key.clone();

            cx.spawn(async move |this, cx| {
                let res = client.get_package_details(&pkg_name).await;
                let _ = this.update(cx, |view, cx| {
                    view.store.update(cx, |st, _cx| match res {
                        Ok(Some(details)) => {
                            st.cache_details(key_clone, details);
                        }
                        Ok(None) => {
                            st.set_detail_error(key_clone, "Package details not found".to_string());
                        }
                        Err(e) => {
                            st.set_detail_error(key_clone, e.to_string());
                        }
                    });
                    cx.notify();
                });
            })
            .detach();
        }
    }

    /// S'assure que la recette PKGBUILD d'un paquet AUR est disponible dans le cache
    fn ensure_pkgbuild(&mut self, pkg_name: String, cx: &mut Context<Self>) {
        if self.store.read(cx).get_cached_pkgbuild(&pkg_name).is_some() {
            return;
        }

        let client = self.store.read(cx).client.clone();
        let name_clone = pkg_name.clone();

        self.workstation.update(cx, |ws, cx| {
            ws.is_loading_pkgbuild = true;
            cx.notify();
        });

        cx.spawn(async move |this, cx| {
            let res = client.fetch_aur_pkgbuild(&name_clone).await;
            let _ = this.update(cx, |view, cx| {
                view.workstation.update(cx, |ws, cx| {
                    ws.is_loading_pkgbuild = false;
                    cx.notify();
                });
                match res {
                    Ok(content) => {
                        view.store.update(cx, |st, _cx| {
                            st.cache_pkgbuild(name_clone, content);
                        });
                    }
                    Err(err) => {
                        log::warn!(
                            "Échec de récupération du PKGBUILD pour {}: {:?}",
                            name_clone,
                            err
                        );
                        let reduce = view.gpui_config.reduce_motion;
                        view.toast_center.update(cx, |tc, cx| {
                            tc.post(
                                ToastKind::Warning,
                                "PKGBUILD unavailable",
                                format!("Failed to load PKGBUILD for {}", name_clone),
                                None,
                                reduce,
                                cx,
                            );
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Déclenche la recherche avec temporisation (debounce)
    fn on_search_input(&mut self, input: String, cx: &mut Context<Self>) {
        self.search_input_buffer = input.clone();

        // Dans Installés et Mises à jour, la requête réduit les lignes déjà
        // chargées : elle n'interroge pas les dépôts.
        if self.session.read(cx).scope != NavScope::Results {
            self.search_debounce_task = None;
            self.session.update(cx, |s, cx| s.set_search_query(input, cx));
            return;
        }

        if input.trim().is_empty() {
            self.search_debounce_task = None;
            self.execute_search(String::new(), cx);
            return;
        }

        self.search_debounce_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(60))
                .await;
            let _ = this.update(cx, |view, cx| {
                view.execute_search(input, cx);
            });
        }));
    }

    /// Réessaye la recherche pour une source spécifique après échec
    fn retry_source_search(&mut self, kind: PackageSourceKind, cx: &mut Context<Self>) {
        let query = self.session.read(cx).search_query.trim().to_string();
        if query.is_empty() {
            return;
        }

        let aur_enabled = self.calix_settings.aur_enabled;
        let flatpak_enabled = self.calix_settings.flat_pack_enabled;
        let appimage_enabled = self.calix_settings.app_image_enabled;
        let client = self.store.read(cx).client.clone();
        let query_clone = query.clone();

        cx.spawn(async move |this, cx| {
            let res: Result<Vec<UnifiedPackage>, String> = match kind {
                PackageSourceKind::Alpm => client
                    .search_standard(&query_clone)
                    .await
                    .map(|pkgs| {
                        pkgs.into_iter()
                            .map(|p| UnifiedPackage::from_alpm(p, false))
                            .collect()
                    })
                    .map_err(|e| e.to_string()),
                PackageSourceKind::Aur => {
                    if aur_enabled {
                        client
                            .search_aur(&query_clone)
                            .await
                            .map(|pkgs| {
                                pkgs.into_iter()
                                    .map(|p| UnifiedPackage::from_aur(p, false))
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(Vec::new())
                    }
                }
                PackageSourceKind::Flatpak => {
                    if flatpak_enabled {
                        client
                            .search_flatpak(&query_clone)
                            .await
                            .map(|pkgs| {
                                pkgs.into_iter()
                                    .map(|p| UnifiedPackage::from_flatpak(p, false))
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(Vec::new())
                    }
                }
                // Les écosystèmes Calix ne sont pas des backends de recherche :
                // leurs paquets sont déjà installés et n'apparaissent que dans
                // l'inventaire, jamais parmi les résultats de recherche.
                PackageSourceKind::Cargo
                | PackageSourceKind::Npm
                | PackageSourceKind::Release => Ok(Vec::new()),
                PackageSourceKind::AppImage => {
                    if appimage_enabled {
                        client
                            .list_appimages()
                            .await
                            .map(|appimages| {
                                let q_lower = query_clone.to_lowercase();
                                appimages
                                    .into_iter()
                                    .filter(|ai| {
                                        ai.name.to_lowercase().contains(&q_lower)
                                            || ai
                                                .desktop_name
                                                .as_ref()
                                                .map(|d| d.to_lowercase().contains(&q_lower))
                                                .unwrap_or(false)
                                            || ai
                                                .description
                                                .as_ref()
                                                .map(|d| d.to_lowercase().contains(&q_lower))
                                                .unwrap_or(false)
                                    })
                                    .map(UnifiedPackage::from_appimage)
                                    .collect()
                            })
                            .map_err(|e| e.to_string())
                    } else {
                        Ok(Vec::new())
                    }
                }
            };

            let _ = this.update(cx, |view, cx| {
                view.store.update(cx, |st, cx| match res {
                    Ok(new_pkgs) => {
                        st.update_source_health(kind, SourceHealth::ready());
                        let mut current: Vec<UnifiedPackage> = st
                            .active_results
                            .iter()
                            .filter(|p| p.key().source != kind)
                            .cloned()
                            .collect();
                        current.extend(new_pkgs);
                        let gen = view.session.read(cx).search_generation;
                        st.set_active_results(current, gen, cx);

                        let scope = view.session.read(cx).source_scope;
                        let all_healthy = scope
                            .enabled_sources(aur_enabled, flatpak_enabled, appimage_enabled)
                            .iter()
                            .all(|src| {
                                st.source_health
                                    .get(src)
                                    .map(|h| !h.is_failed())
                                    .unwrap_or(true)
                            });
                        if all_healthy {
                            st.cache_search(&query_clone, scope, st.active_results.to_vec());
                        }
                    }
                    Err(err) => {
                        st.update_source_health(kind, SourceHealth::failed(err));
                    }
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Exécute la recherche unifiée multi-sources avec cache de session et protection contre les requêtes obsolètes
    fn execute_search(&mut self, query: String, cx: &mut Context<Self>) {
        let trimmed = query.trim().to_string();

        if trimmed.is_empty() {
            let next_gen = self.session.update(cx, |s, cx| {
                s.search_query.clear();
                s.set_searching(false, cx);
                s.select_package(None, cx);
                s.next_search_generation()
            });
            self.store.update(cx, |st, cx| {
                st.set_active_results(Vec::new(), next_gen, cx);
            });
            return;
        }

        let source_scope = self.session.read(cx).source_scope;

        // 1. Vérification du cache de session
        if let Some(cached) = self
            .store
            .read(cx)
            .get_cached_search(&trimmed, source_scope)
            .cloned()
        {
            let gen = self.session.update(cx, |s, _cx| {
                s.search_query = trimmed.clone();
                s.is_searching = false;
                s.next_search_generation()
            });
            self.store.update(cx, |st, cx| {
                st.set_active_results(cached, gen, cx);
            });
            self.workstation.update(cx, |ws, _cx| {
                ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
            });
            return;
        }

        // 2. Requête backend concurrente
        let gen = self.session.update(cx, |s, _cx| {
            s.search_query = trimmed.clone();
            s.is_searching = true;
            s.next_search_generation()
        });

        let aur_enabled = self.calix_settings.aur_enabled;
        let flatpak_enabled = self.calix_settings.flat_pack_enabled;
        let appimage_enabled = self.calix_settings.app_image_enabled;

        let client = self.store.read(cx).client.clone();
        let query_clone = trimmed.clone();

        cx.spawn(async move |this, cx| {
            let (alpm_res, aur_res, flatpak_res, appimage_res) = tokio::join!(
                async {
                    if source_scope.alpm {
                        Some(client.search_standard(&query_clone).await)
                    } else {
                        None
                    }
                },
                async {
                    if source_scope.aur && aur_enabled {
                        Some(client.search_aur(&query_clone).await)
                    } else {
                        None
                    }
                },
                async {
                    if source_scope.flatpak && flatpak_enabled {
                        Some(client.search_flatpak(&query_clone).await)
                    } else {
                        None
                    }
                },
                async {
                    if source_scope.appimage && appimage_enabled {
                        Some(client.list_appimages().await)
                    } else {
                        None
                    }
                }
            );

            let _ = this.update(cx, |view, cx| {
                let current_gen = view.session.read(cx).search_generation;
                if current_gen != gen {
                    return;
                }

                let mut combined = Vec::new();
                let mut all_succeeded = true;

                view.store.update(cx, |st, cx| {
                    if let Some(res) = alpm_res {
                        match res {
                            Ok(pkgs) => {
                                st.update_source_health(
                                    PackageSourceKind::Alpm,
                                    SourceHealth::ready(),
                                );
                                for p in pkgs {
                                    combined.push(UnifiedPackage::from_alpm(p, false));
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::Alpm,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(res) = aur_res {
                        match res {
                            Ok(pkgs) => {
                                st.update_source_health(
                                    PackageSourceKind::Aur,
                                    SourceHealth::ready(),
                                );
                                for p in pkgs {
                                    combined.push(UnifiedPackage::from_aur(p, false));
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::Aur,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(res) = flatpak_res {
                        match res {
                            Ok(pkgs) => {
                                st.update_source_health(
                                    PackageSourceKind::Flatpak,
                                    SourceHealth::ready(),
                                );
                                for p in pkgs {
                                    combined.push(UnifiedPackage::from_flatpak(p, false));
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::Flatpak,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if let Some(res) = appimage_res {
                        match res {
                            Ok(appimages) => {
                                st.update_source_health(
                                    PackageSourceKind::AppImage,
                                    SourceHealth::ready(),
                                );
                                let q_lower = query_clone.to_lowercase();
                                for ai in appimages {
                                    let matches = ai.name.to_lowercase().contains(&q_lower)
                                        || ai
                                            .desktop_name
                                            .as_ref()
                                            .map(|d| d.to_lowercase().contains(&q_lower))
                                            .unwrap_or(false)
                                        || ai
                                            .description
                                            .as_ref()
                                            .map(|d| d.to_lowercase().contains(&q_lower))
                                            .unwrap_or(false);
                                    if matches {
                                        combined.push(UnifiedPackage::from_appimage(ai));
                                    }
                                }
                            }
                            Err(e) => {
                                all_succeeded = false;
                                st.update_source_health(
                                    PackageSourceKind::AppImage,
                                    SourceHealth::failed(e.to_string()),
                                );
                            }
                        }
                    }

                    if all_succeeded {
                        st.cache_search(&query_clone, source_scope, combined.clone());
                    }

                    st.set_active_results(combined, gen, cx);
                });

                view.session.update(cx, |s, cx| {
                    s.set_searching(false, cx);
                });
                view.workstation.update(cx, |ws, _cx| {
                    ws.scroll_handle.scroll_to_item(0, ScrollStrategy::Top)
                });
                cx.notify();
            });
        })
        .detach();
    }

    /// Exécute une opération de mutation (installation, suppression, mise à jour) avec streaming
    fn run_package_mutation(
        &mut self,
        action_name: &str,
        action: MutationAction,
        name: String,
        key: PackageKey,
        cx: &mut Context<Self>,
    ) {
        if self.console.read(cx).is_running() {
            return;
        }

        self.console.update(cx, |c, cx| {
            c.start_operation(action_name, cx);
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<LogStreamEvent>();
        let client = self.store.read(cx).client.clone();

        match action {
            MutationAction::Install { is_aur, is_flatpak } => {
                client.install_package(&name, is_aur, is_flatpak, tx);
            }
            MutationAction::Remove { is_flatpak } => {
                let cascade = self.calix_settings.package_management_cascade_delete;
                let remove_configs = self.calix_settings.package_management_remove_configs;
                client.remove_package(&name, is_flatpak, cascade, remove_configs, tx);
            }
            MutationAction::UpgradeSystem => {
                client.upgrade_system(tx);
            }
        }

        cx.spawn(async move |this, cx| {
            let mut final_status = false;
            let mut final_msg = String::from("Operation completed");

            while let Some(event) = rx.recv().await {
                match event {
                    LogStreamEvent::Line(line) => {
                        let _ = this.update(cx, |view, cx| {
                            view.console.update(cx, |c, cx| c.append_stdout(&line, cx));
                        });
                    }
                    LogStreamEvent::ErrorLine(line) => {
                        let _ = this.update(cx, |view, cx| {
                            view.console.update(cx, |c, cx| c.append_stderr(&line, cx));
                        });
                    }
                    LogStreamEvent::Finished(success, code) => {
                        final_status = success;
                        final_msg = if success {
                            format!("Success (exit code {:?})", code)
                        } else {
                            format!("Operation failed (exit code {:?})", code)
                        };
                    }
                }
            }

            let _ = this.update(cx, |view, cx| {
                view.console.update(cx, |c, cx| {
                    c.finish_operation(final_status, &final_msg, cx);
                });

                if final_status {
                    view.store.update(cx, |st, cx| {
                        st.invalidate_package(&key, cx);
                        st.invalidate_installed(cx);
                        st.invalidate_updates(cx);
                    });

                    let scope = view.session.read(cx).scope;
                    match scope {
                        NavScope::Installed => view.load_installed_packages(cx),
                        NavScope::Updates => view.load_updates(cx),
                        NavScope::Results => {}
                    }
                }
            });
        })
        .detach();
    }

    /// Sauvegarde les paramètres modifiés et met à jour l'application
    pub fn save_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let bounds = window.window_bounds();
        if let WindowBounds::Windowed(b) = bounds {
            let raw_w = b.size.width / px(1.0);
            let raw_h = b.size.height / px(1.0);
            let (w, h) = ConfigManager::sanitize_window_size(raw_w, raw_h);
            self.settings.draft_gpui.window_width = w;
            self.settings.draft_gpui.window_height = h;
        }

        self.settings.draft_gpui.last_scope = self.session.read(cx).scope;

        let draft_aur = self.settings.draft_calix.aur_enabled;
        let draft_flatpak = self.settings.draft_calix.flat_pack_enabled;
        let draft_appimage = self.settings.draft_calix.app_image_enabled;

        let mut scope = self.session.read(cx).source_scope;
        let mut scope_changed = false;
        if !draft_aur && scope.aur {
            scope.aur = false;
            scope_changed = true;
        }
        if !draft_flatpak && scope.flatpak {
            scope.flatpak = false;
            scope_changed = true;
        }
        if !draft_appimage && scope.appimage {
            scope.appimage = false;
            scope_changed = true;
        }

        if scope_changed {
            self.session.update(cx, |s, cx| {
                s.set_source_scope(scope, cx);
            });
        }

        match self.settings.save() {
            Ok(()) => {
                let sources_changed = self.calix_settings.aur_enabled != draft_aur
                    || self.calix_settings.flat_pack_enabled != draft_flatpak
                    || self.calix_settings.app_image_enabled != draft_appimage;

                self.calix_settings = self.settings.draft_calix.clone();
                self.gpui_config = self.settings.draft_gpui.clone();
                self.theme = if self.gpui_config.dark_theme {
                    Theme::dark()
                } else {
                    Theme::light()
                }
                .with_brand(self.gpui_config.brand_color);

                let theme = self.theme;
                self.motion_policy =
                    crate::state::motion::MotionPolicy::new(self.gpui_config.reduce_motion);
                let reduce = self.motion_policy.reduce_motion;

                self.workstation.update(cx, |ws, cx| {
                    ws.set_theme(theme, cx);
                    ws.set_visual_style(self.gpui_config.visual_style, cx);
                    ws.set_reduce_motion(reduce, cx);
                    ws.set_compact(self.gpui_config.compact_view, cx);
                    ws.set_sources_enabled(draft_aur, draft_flatpak, draft_appimage, cx);
                });

                if sources_changed {
                    self.session.update(cx, |s, cx| {
                        s.clamp_source_scope(draft_aur, draft_flatpak, draft_appimage, cx);
                    });
                }

                self.sidebar.update(cx, |sb, cx| {
                    sb.set_theme(theme, cx);
                    sb.set_visual_style(self.gpui_config.visual_style, cx);
                    sb.set_reduce_motion(reduce, cx);
                });

                self.console_view.update(cx, |cv, cx| {
                    cv.set_theme(theme, cx);
                    cv.set_reduce_motion(reduce, cx);
                    cv.set_configured_height(self.gpui_config.log_drawer_height, cx);
                });

                self.console.update(cx, |c, _| {
                    c.set_auto_open(self.gpui_config.log_drawer_open);
                });

                if sources_changed {
                    self.store.update(cx, |st, _| {
                        st.search_cache.clear();
                    });
                    let query = self.session.read(cx).search_query.clone();
                    if !query.trim().is_empty() {
                        self.execute_search(query, cx);
                    }
                }

                self.toast_center.update(cx, |tc, cx| {
                    tc.post(
                        ToastKind::Success,
                        "Settings saved",
                        "Your preferences have been saved to disk.",
                        None,
                        reduce,
                        cx,
                    );
                });
                cx.notify();
            }
            Err(e) => {
                let reduce = self.gpui_config.reduce_motion;
                self.toast_center.update(cx, |tc, cx| {
                    tc.post(
                        ToastKind::Error,
                        "Failed to save settings",
                        format!("{}", e),
                        Some(ToastAction::OpenLogs),
                        reduce,
                        cx,
                    );
                });
                cx.notify();
            }
        }
    }

    /// Récupère la liste de paquets actuellement affichée selon le périmètre actif
    fn current_display_packages<'a>(&self, cx: &'a Context<Self>) -> &'a [UnifiedPackage] {
        let scope = self.session.read(cx).scope;
        let store = self.store.read(cx);
        match scope {
            NavScope::Results => &store.active_results,
            NavScope::Installed => &store.installed_packages,
            NavScope::Updates => &store.updates_packages,
        }
    }

    /// Navigation au clavier dans la liste virtuelle (flèches Haut/Bas)
    pub fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;

        // ── Les touches globales ─────────────────────────────────────────────
        // ⌘1 à ⌘5 : les cinq entrées du rail. ⌥← et ⌥→ : le périmètre
        // précédent et le suivant. ⌘F : la recherche. ⌘L : la console.
        // ⌘⌥C : le rail.
        if modifiers.platform {
            let rail_target = match key {
                "1" => Some((Place::Packages, Some(NavScope::Results))),
                "2" => Some((Place::Packages, Some(NavScope::Installed))),
                "3" => Some((Place::Packages, Some(NavScope::Updates))),
                "4" => Some((Place::News, None)),
                "5" => Some((Place::Settings, None)),
                _ => None,
            };
            if let Some((place, scope)) = rail_target {
                self.session.update(cx, |s, cx| {
                    if let Some(scope) = scope {
                        s.set_scope(scope, cx);
                    }
                    s.set_place(place, cx);
                });
                return;
            }
            if key == "f" {
                let search_input = self.workstation.read(cx).search_input.clone();
                search_input.update(cx, |si, _cx| si.focus(window));
                return;
            }
            if key == "l" {
                self.console.update(cx, |c, cx| c.toggle_drawer(cx));
                return;
            }
            if key == "c" && modifiers.alt {
                self.session.update(cx, |s, cx| s.toggle_sidebar(cx));
                return;
            }
        }

        if modifiers.alt && !modifiers.platform {
            let scope = self.session.read(cx).scope;
            let next = match key {
                "left" => scope.previous(),
                "right" => scope.next(),
                _ => None,
            };
            if let Some(next) = next {
                self.session.update(cx, |s, cx| {
                    s.set_scope(next, cx);
                    s.set_place(Place::Packages, cx);
                });
                return;
            }
        }

        if key == "escape" {
            // Échap est le retour local : il efface la recherche, puis il
            // quitte le lieu, puis il vide la sélection.
            let search_input = self.workstation.read(cx).search_input.clone();
            let query = self.session.read(cx).search_query.clone();
            if search_input.read(cx).is_focused(window) && !query.trim().is_empty() {
                search_input.update(cx, |si, cx| si.set_text(String::new(), cx));
                self.session.update(cx, |s, cx| s.set_search_query(String::new(), cx));
                self.on_search_input(String::new(), cx);
                return;
            }
            let place = self.session.read(cx).place;
            if place != Place::Packages {
                self.session.update(cx, |s, cx| s.set_place(Place::Packages, cx));
                return;
            }
            self.session.update(cx, |s, cx| s.select_package(None, cx));
            return;
        }

        if key == "tab" {
            if event.keystroke.modifiers.shift {
                window.focus_prev();
            } else {
                window.focus_next();
            }
            return;
        }

        if key == "/" && !event.keystroke.modifiers.control && !event.keystroke.modifiers.alt {
            let search_input = self.workstation.read(cx).search_input.clone();
            let is_focused = search_input.read(cx).is_focused(window);
            if !is_focused {
                search_input.update(cx, |si, _cx| si.focus(window));
                return;
            }
        }

        let packages = self.current_display_packages(cx);
        if packages.is_empty() {
            return;
        }

        let selected_key = self.session.read(cx).selected_package_key.clone();

        let current_idx = selected_key
            .as_ref()
            .and_then(|sk| packages.iter().position(|p| &p.key() == sk));

        match key {
            "down" => {
                let next_idx = match current_idx {
                    Some(idx) => (idx + 1).min(packages.len().saturating_sub(1)),
                    None => 0,
                };
                let next_pkg = &packages[next_idx];
                let next_key = next_pkg.key();
                self.session.update(cx, |s, cx| {
                    s.select_package(Some(next_key), cx);
                });
                self.workstation.update(cx, |ws, _cx| {
                    ws.scroll_handle
                        .scroll_to_item_strict(next_idx, ScrollStrategy::Top)
                });
            }
            "up" => {
                let prev_idx = match current_idx {
                    Some(idx) => idx.saturating_sub(1),
                    None => 0,
                };
                let prev_pkg = &packages[prev_idx];
                let prev_key = prev_pkg.key();
                self.session.update(cx, |s, cx| {
                    s.select_package(Some(prev_key), cx);
                });
                self.workstation.update(cx, |ws, _cx| {
                    ws.scroll_handle
                        .scroll_to_item_strict(prev_idx, ScrollStrategy::Top)
                });
            }
            "escape" => {
                self.session.update(cx, |s, cx| {
                    s.select_package(None, cx);
                });
            }
            _ => {}
        }
    }
    /// La surface de développement : le laboratoire de rendu.
    ///
    /// Ce n'est pas un lieu : il s'ouvre par-dessus, et le lieu d'en dessous
    /// est retrouvé à la fermeture.
    fn render_lab_content(
        &self,
        cx: &mut Context<Self>,
        theme: &Theme,
        entity: &Entity<Self>,
    ) -> AnyElement {
            let entity_rl = entity.clone();
            let rl_state = self.render_lab.read(cx);
            div()
                .size_full()
                .child(crate::views::render_lab::RenderLabView::render(
                    crate::views::render_lab::RenderLabViewProps {
                        state: rl_state,
                        theme: &theme,
                        ledger_collapsed: self.render_lab_ledger_collapsed,
                        catalog_scroll_handle: &self.render_lab_catalog_scroll,
                        on_select_fixture: {
                            let e = entity_rl.clone();
                            Rc::new(move |id, _w, cx| {
                                e.update(cx, |view, cx| {
                                    view.render_lab.update(cx, |rl, cx| {
                                        rl.set_fixture(Some(id), cx);
                                    });
                                    cx.notify();
                                });
                            })
                        },
                        on_clear_fixture: {
                            let e = entity_rl.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.render_lab.update(cx, |rl, cx| {
                                        rl.set_fixture(None, cx);
                                    });
                                    cx.notify();
                                });
                            })
                        },
                        on_back_settings: {
                            let e = entity_rl.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.session.update(cx, |s, cx| {
                                        s.set_render_lab(false, cx);
                                        s.set_place(Place::Settings, cx);
                                    });
                                });
                            })
                        },
                        on_toggle_ledger: {
                            let e = entity_rl;
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.render_lab_ledger_collapsed =
                                        !view.render_lab_ledger_collapsed;
                                    cx.notify();
                                });
                            })
                        },
                    },
                ))
            .into_any_element()
    }
}
impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let entity = cx.entity().clone();

        // ── Lecture de l'état de session ────────────────────────────────────
        let (place, render_lab, navigation_epoch) = {
            let session = self.session.read(cx);
            (session.place, session.render_lab, session.navigation_epoch)
        };

        // ── 1. Contenu principal selon le lieu ──────────────────────────────
        // Le laboratoire de rendu est une surface de développement, pas un
        // lieu : il s'ouvre par-dessus et le lieu d'en dessous est retrouvé.
        let main_content = if render_lab {
            self.render_lab_content(cx, &theme, &entity)
        } else {
            match place {
            Place::Packages => {
                div().size_full().child(self.workstation.clone())
            }
            Place::News => {
                let entity_retry = entity.clone();
                let store = self.store.read(cx);
                div().size_full().child(NewsView::render(NewsViewProps {
                    news: &self.news,
                    error: store.news_error.as_deref(),
                    is_loading: self.is_loading_news,
                    theme: &theme,
                    on_retry: Some(Rc::new(move |_w, cx| {
                        entity_retry.update(cx, |view, cx| {
                            view.load_news(cx);
                        });
                    })),
                }))
            }
            Place::Settings => {
                let entity_st = entity.clone();
                let theme_ref = &theme;
                div()
                    .size_full()
                    .child(SettingsView::render(SettingsViewProps {
                        calix_settings: &self.settings.draft_calix,
                        gpui_config: &self.settings.draft_gpui,
                        is_dirty: self.settings.is_dirty,
                        style_menu_open: self.settings.style_menu_open,
                        style_menu_epoch: self.settings.style_menu_epoch,
                        style_menu_highlighted: self.settings.style_menu_highlighted,
                        advanced_settings_expanded: self.settings.advanced_settings_expanded,
                        theme: theme_ref,
                        on_toggle_aur: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_aur();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_flatpak: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_flatpak();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_appimage: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_appimage();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_cascade_delete: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_cascade_delete();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_remove_configs: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_remove_configs();
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_dark_theme: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_dark_theme();
                                    let theme = if view.settings.draft_gpui.dark_theme {
                                        Theme::dark()
                                    } else {
                                        Theme::light()
                                    }
                                    .with_brand(view.settings.draft_gpui.brand_color);
                                    view.theme = theme;
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_theme(theme, cx));
                                    view.sidebar.update(cx, |sb, cx| sb.set_theme(theme, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_theme(theme, cx));
                                    cx.notify();
                                })
                            })
                        },
                        on_select_brand: {
                            let e = entity_st.clone();
                            Rc::new(move |brand: crate::brand::Brand, _w, cx| {
                                e.update(cx, |view, cx| {
                                    // The colour is applied to the draft and to the
                                    // theme at once: the accent changes under the
                                    // user's finger, the save commits it.
                                    view.settings.set_brand_color(brand);
                                    let theme = if view.settings.draft_gpui.dark_theme {
                                        Theme::dark()
                                    } else {
                                        Theme::light()
                                    }
                                    .with_brand(brand);
                                    view.theme = theme;
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_theme(theme, cx));
                                    view.sidebar.update(cx, |sb, cx| sb.set_theme(theme, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_theme(theme, cx));
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_compact_view: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_compact_view();
                                    let compact = view.settings.draft_gpui.compact_view;
                                    view.session
                                        .update(cx, |s, cx| s.set_sidebar_collapsed(compact, cx));
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_compact(compact, cx));
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_log_drawer_auto_open: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_log_drawer_auto_open();
                                    let auto_open = view.settings.draft_gpui.log_drawer_open;
                                    view.console.update(cx, |c, _| c.set_auto_open(auto_open));
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_reduce_motion: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_reduce_motion();
                                    let live_reduce = view.settings.draft_gpui.reduce_motion;
                                    view.gpui_config.reduce_motion = live_reduce;
                                    view.motion_policy =
                                        crate::state::motion::MotionPolicy::new(live_reduce);
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_reduce_motion(live_reduce, cx));
                                    view.sidebar
                                        .update(cx, |sb, cx| sb.set_reduce_motion(live_reduce, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_reduce_motion(live_reduce, cx));
                                    cx.notify();
                                })
                            })
                        },
                        on_open_style_menu: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.open_style_menu();
                                    cx.notify();
                                })
                            })
                        },
                        on_close_style_menu: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.close_style_menu();
                                    cx.notify();
                                })
                            })
                        },
                        on_navigate_style_menu: {
                            let e = entity_st.clone();
                            Rc::new(move |key: &str, _w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.navigate_style_menu(key);
                                    cx.notify();
                                })
                            })
                        },
                        on_select_style: {
                            let e = entity_st.clone();
                            Rc::new(move |id: VisualStyleId, _w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.set_visual_style(id);
                                    cx.notify();
                                })
                            })
                        },
                        on_toggle_advanced_settings: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.toggle_advanced_settings();
                                    cx.notify();
                                })
                            })
                        },
                        on_open_render_lab: {
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.session.update(cx, |s, cx| {
                                        s.set_render_lab(true, cx);
                                    });
                                })
                            })
                        },
                        on_save: {
                            let e = entity_st.clone();
                            Rc::new(move |w, cx| {
                                e.update(cx, |view, cx| {
                                    view.save_settings(w, cx);
                                })
                            })
                        },
                        on_reset: Some({
                            let e = entity_st.clone();
                            Rc::new(move |_w, cx| {
                                e.update(cx, |view, cx| {
                                    view.settings.reset_to(
                                        view.calix_settings.clone(),
                                        view.gpui_config.clone(),
                                    );
                                    let theme = if view.gpui_config.dark_theme {
                                        Theme::dark()
                                    } else {
                                        Theme::light()
                                    }
                                    .with_brand(view.gpui_config.brand_color);
                                    view.theme = theme;
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_theme(theme, cx));
                                    view.sidebar.update(cx, |sb, cx| sb.set_theme(theme, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_theme(theme, cx));
                                    let compact = view.gpui_config.compact_view;
                                    view.session
                                        .update(cx, |s, cx| s.set_sidebar_collapsed(compact, cx));
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_compact(compact, cx));
                                    let auto_open = view.gpui_config.log_drawer_open;
                                    view.console.update(cx, |c, _| c.set_auto_open(auto_open));
                                    let reduce = view.gpui_config.reduce_motion;
                                    view.motion_policy =
                                        crate::state::motion::MotionPolicy::new(reduce);
                                    view.workstation
                                        .update(cx, |ws, cx| ws.set_reduce_motion(reduce, cx));
                                    view.sidebar
                                        .update(cx, |sb, cx| sb.set_reduce_motion(reduce, cx));
                                    view.console_view
                                        .update(cx, |cv, cx| cv.set_reduce_motion(reduce, cx));
                                    cx.notify();
                                })
                            })
                        }),
                    }))
            }
            }
            .into_any_element()
        };

        // Transition discrète de destination (120ms d'opacité ou immédiat sous reduce_motion)
        let reduce_motion = self.motion_policy.reduce_motion;
        let destination_element = if reduce_motion {
            div()
                .id("dest_view")
                .size_full()
                .child(main_content)
                .into_any_element()
        } else {
            div()
                .id("dest_view")
                .size_full()
                .child(main_content)
                .with_animation(
                    ("nav-fade", navigation_epoch as usize),
                    Animation::new(MotionDurations::FAST).with_easing(gpui::ease_out_quint()),
                    |elem, delta| elem.opacity(delta),
                )
                .into_any_element()
        };

        // ── 2. Overlay de notifications (Toasts) ────────────────────────────
        let toasts = self.toast_center.read(cx).toasts.clone();
        let entity_toast_dismiss = entity.clone();
        let entity_toast_action = entity.clone();

        let toast_overlay = ToastOverlay::render(ToastOverlayProps {
            toasts: &toasts,
            reduce_motion,
            theme: &theme,
            on_dismiss: Rc::new(move |id, _w, cx| {
                entity_toast_dismiss.update(cx, |view, cx| {
                    let reduce = view.gpui_config.reduce_motion;
                    view.toast_center.update(cx, |tc, cx| {
                        tc.dismiss(id, reduce, cx);
                    });
                });
            }),
            on_action: Rc::new(move |action, _w, cx| {
                entity_toast_action.update(cx, |view, cx| match action {
                    ToastAction::OpenLogs => {
                        view.console.update(cx, |c, cx| c.set_open(true, cx));
                    }
                });
            }),
        });

        // ── 3. L'en-tête de contexte ─────────────────────────────────────────
        // Il dit où l'on est : le lieu, le périmètre, le compte, la couverture
        // des écosystèmes, l'âge de la réponse et les filtres en force.
        let (header_place, header_scope, source_scope, state_filter) = {
            let session = self.session.read(cx);
            (
                session.place,
                session.scope,
                session.source_scope,
                session.state_filter,
            )
        };
        // The count is the place's: the scope's rows, or the announcements.
        let visible_count = match header_place {
            Place::Packages => {
                let store = self.store.read(cx);
                let rows = match header_scope {
                    NavScope::Results => &store.active_results,
                    NavScope::Installed => &store.installed_packages,
                    NavScope::Updates => &store.updates_packages,
                };
                crate::state::query::visible_packages(
                    rows,
                    &source_scope,
                    state_filter,
                    SortMode::Relevance,
                )
                .len()
            }
            Place::News => self.news.len(),
            Place::Settings => 0,
        };
        let is_searching = self.session.read(cx).is_searching;
        let aur_enabled = self.calix_settings.aur_enabled;
        let flatpak_enabled = self.calix_settings.flat_pack_enabled;
        let appimage_enabled = self.calix_settings.app_image_enabled;

        let entity_clear = self.session.clone();
        let context_header = ContextHeader::render(ContextHeaderProps {
            place: header_place,
            scope: header_scope,
            row_count: visible_count,
            row_count_unknown: is_searching,
            source_scope,
            state_filter,
            aur_enabled,
            flatpak_enabled,
            appimage_enabled,
            age: None,
            source_health: &self.store.read(cx).source_health,
            on_retry_source: {
                let entity_retry = entity.clone();
                Rc::new(move |kind: PackageSourceKind, _w, cx| {
                    entity_retry.update(cx, |view, cx| {
                        view.retry_source_search(kind, cx);
                    });
                })
            },
            on_clear_filters: Rc::new(move |_w, cx| {
                entity_clear.update(cx, |s, cx| {
                    s.clear_filters(aur_enabled, flatpak_enabled, appimage_enabled, cx);
                });
            }),
            theme: &theme,
        });

        // ── 4. Assemblage final du shell ─────────────────────────────────────
        let entity_key = entity.clone();

        div()
            .id("workspace_root")
            .size_full()
            .relative()
            .flex()
            .flex_row()
            .bg(theme.bg_app)
            .text_color(theme.text_primary)
            .track_focus(&self.search_focus)
            .on_key_down(move |event, window, cx| {
                entity_key.update(cx, |view, cx| {
                    view.on_key_down(event, window, cx);
                });
            })
            .child(self.sidebar.clone())
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(context_header)
                    .child(
                        div()
                            .flex_1()
                            .min_h(px(0.0))
                            .overflow_hidden()
                            .child(destination_element),
                    )
                    .child(self.console_view.clone()),
            )
            .child(toast_overlay)
    }
}

/// Fusionne la liste typée d'ALPM avec l'inventaire que sert le daemon.
///
/// Un daemon absent n'est pas un échec de chargement : la fenêtre a encore
/// des paquets à montrer, et le journal dit pourquoi les autres écosystèmes
/// manquent. Un frontend qui refuse de démarrer parce qu'un second service
/// est arrêté est pire qu'un frontend incomplet.
fn merged_installed(
    installed: Vec<crate::backend::models::AlpmPackage>,
    snapshot: &Result<crate::backend::calix_snapshot::Snapshot, String>,
) -> Vec<crate::backend::models::UnifiedPackage> {
    match snapshot {
        Ok(snapshot) => crate::backend::models::merge_snapshot(installed, snapshot),
        Err(error) => {
            log::warn!("Calix daemon unavailable, showing backend packages only: {error}");
            crate::backend::models::merge_snapshot(
                installed,
                &crate::backend::calix_snapshot::Snapshot::default(),
            )
        }
    }
}
