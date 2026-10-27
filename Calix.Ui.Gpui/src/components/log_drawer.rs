use crate::icons::AppIcon;
use crate::theme::Theme;
use gpui::*;
use gpui::prelude::FluentBuilder;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Idle,
    Running(String),
    Success(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendPhaseStatus {
    Pending,
    Running(Option<u8>),
    Success(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackendPhase {
    pub name: String,
    pub status: BackendPhaseStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub text: String,
    pub is_stderr: bool,
    pub raw: Option<String>,
}

impl LogEntry {
    pub fn decoded(text: impl Into<String>, is_stderr: bool, raw: Option<String>) -> Self {
        let t: String = text.into();
        Self {
            text: sanitize_ansi(&t),
            is_stderr,
            raw,
        }
    }
}

pub type LogActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct LogDrawerProps<'a> {
    /// The product's own command, the one it shows. The log underneath shows
    /// what the backend actually ran.
    pub product_command: Option<&'a str>,
    pub logs: &'a [LogEntry],
    pub backend_phases: &'a [BackendPhase],
    pub status: &'a OperationStatus,
    pub is_open: bool,
    pub auto_scroll: bool,
    pub show_raw_logs: bool,
    pub height: f32,
    /// Le tiroir passe par-dessus le catalogue : il garde sa hauteur entière,
    /// sans plafond relatif.
    pub overlays: bool,
    pub copied_feedback: bool,
    pub scroll_handle: &'a ScrollHandle,
    pub theme: &'a Theme,
    pub on_toggle: Option<LogActionHandler>,
    pub on_copy: Option<LogActionHandler>,
    pub on_clear: Option<LogActionHandler>,
    pub on_toggle_autoscroll: Option<LogActionHandler>,
    pub on_toggle_raw_logs: Option<LogActionHandler>,
}

pub struct LogDrawer;

impl LogDrawer {
    pub fn render(props: LogDrawerProps) -> impl IntoElement {
        let theme = props.theme;
        let is_open = props.is_open;
        let auto_scroll = props.auto_scroll;
        let copied_feedback = props.copied_feedback;

        let status_badge = match props.status {
            OperationStatus::Idle => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.border)
                .text_xs()
                .text_color(theme.text_muted)
                .child("Ready"),
            OperationStatus::Running(op) => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.warning)
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.bg_app)
                .child(format!("Running: {}", op)),
            OperationStatus::Success(msg) => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.success)
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.bg_app)
                .child(format!("Success: {}", msg)),
            OperationStatus::Error(err) => div()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme.danger)
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.bg_app)
                .child(format!("Error: {}", err)),
        };

        // Bouton Copier les logs
        let copy_btn = {
            let on_copy = props.on_copy.clone();
            let on_copy_key = props.on_copy.clone();
            let hover_bg = theme.bg_surface_hover;
            let border_focus = theme.border_focus;
            let mut btn = div()
                .id("console_copy_logs_btn")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(border_focus))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        if let Some(ref handler) = on_copy_key {
                            handler(window, cx);
                        }
                    }
                })
                .flex()
                .items_center()
                .gap_1p5()
                .px_2()
                .py_1()
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(if copied_feedback {
                    theme.success
                } else {
                    theme.bg_surface
                })
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if copied_feedback {
                    theme.bg_app
                } else {
                    theme.text_primary
                })
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg).border_color(border_focus))
                .child(
                    svg()
                        .path(if copied_feedback {
                            AppIcon::Check.path()
                        } else {
                            AppIcon::Copy.path()
                        })
                        .size_3()
                        .text_color(if copied_feedback {
                            theme.bg_app
                        } else {
                            theme.text_primary
                        }),
                )
                .child(if copied_feedback {
                    "Copied!"
                } else {
                    "Copy logs"
                });

            if let Some(handler) = on_copy {
                btn = btn.on_mouse_down(MouseButton::Left, move |_e, w, cx| handler(w, cx));
            }
            btn
        };

        // Bouton Effacer
        let clear_btn = {
            let on_clear = props.on_clear.clone();
            let on_clear_key = props.on_clear.clone();
            let hover_bg = theme.bg_surface_hover;
            let border_focus = theme.border_focus;
            let mut btn = div()
                .id("console_clear_btn")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(border_focus))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        if let Some(ref handler) = on_clear_key {
                            handler(window, cx);
                        }
                    }
                })
                .flex()
                .items_center()
                .gap_1p5()
                .px_2()
                .py_1()
                .rounded_sm()
                .border_1()
                .border_color(theme.border)
                .bg(theme.bg_surface)
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_secondary)
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg).border_color(border_focus))
                .child(
                    svg()
                        .path(AppIcon::Trash.path())
                        .size_3()
                        .text_color(theme.text_secondary),
                )
                .child("Clear");

            if let Some(handler) = on_clear {
                btn = btn.on_mouse_down(MouseButton::Left, move |_e, w, cx| handler(w, cx));
            }
            btn
        };

        // Bouton Défilement automatique
        let autoscroll_btn = {
            let on_toggle_autoscroll = props.on_toggle_autoscroll.clone();
            let on_autoscroll_key = props.on_toggle_autoscroll.clone();
            let hover_bg = theme.bg_surface_hover;
            let border_focus = theme.border_focus;
            let mut btn = div()
                .id("console_autoscroll_toggle")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(border_focus))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        if let Some(ref handler) = on_autoscroll_key {
                            handler(window, cx);
                        }
                    }
                })
                .px_2()
                .py_1()
                .rounded_sm()
                .border_1()
                .border_color(if auto_scroll {
                    theme.accent
                } else {
                    theme.border
                })
                .bg(if auto_scroll {
                    theme.bg_surface_active
                } else {
                    theme.bg_surface
                })
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if auto_scroll {
                    theme.accent
                } else {
                    theme.text_muted
                })
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg).border_color(border_focus))
                .child(if auto_scroll {
                    "Auto-scroll: ON"
                } else {
                    "Auto-scroll: PAUSE"
                });

            if let Some(handler) = on_toggle_autoscroll {
                btn = btn.on_mouse_down(MouseButton::Left, move |_e, w, cx| handler(w, cx));
            }
            btn
        };

        // Bouton Mode Brut / Formaté
        let raw_toggle_btn = {
            let on_toggle_raw = props.on_toggle_raw_logs.clone();
            let on_raw_key = props.on_toggle_raw_logs.clone();
            let hover_bg = theme.bg_surface_hover;
            let border_focus = theme.border_focus;
            let show_raw = props.show_raw_logs;
            let mut btn = div()
                .id("console_raw_toggle_btn")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(border_focus))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        if let Some(ref handler) = on_raw_key {
                            handler(window, cx);
                        }
                    }
                })
                .px_2()
                .py_1()
                .rounded_sm()
                .border_1()
                .border_color(if show_raw { theme.accent } else { theme.border })
                .bg(if show_raw {
                    theme.bg_surface_active
                } else {
                    theme.bg_surface
                })
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if show_raw {
                    theme.accent
                } else {
                    theme.text_secondary
                })
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg).border_color(border_focus))
                .child(if show_raw {
                    "Logs: Raw"
                } else {
                    "Logs: Formatted"
                });

            if let Some(handler) = on_toggle_raw {
                btn = btn.on_mouse_down(MouseButton::Left, move |_e, w, cx| handler(w, cx));
            }
            btn
        };

        // Bouton de masquage/affichage du tiroir
        let toggle_btn = {
            let on_toggle = props.on_toggle.clone();
            let on_toggle_key = props.on_toggle.clone();
            let border_focus = theme.border_focus;
            let mut btn = div()
                .id("console_toggle_btn")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(border_focus))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        if let Some(ref handler) = on_toggle_key {
                            handler(window, cx);
                        }
                    }
                })
                .flex()
                .items_center()
                .gap_1p5()
                .px_2p5()
                .py_1()
                .rounded_sm()
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_secondary)
                .hover(|s| s.text_color(theme.accent))
                .child(
                    svg()
                        .path(if is_open {
                            AppIcon::Collapse.path()
                        } else {
                            AppIcon::Expand.path()
                        })
                        .size_3()
                        .text_color(theme.text_secondary),
                )
                .child(if is_open { "Hide" } else { "Show" });

            if let Some(handler) = on_toggle {
                btn = btn.on_mouse_down(MouseButton::Left, move |_e, w, cx| handler(w, cx));
            }
            btn
        };

        // Header du tiroir : Seuls les boutons et le chevron de masquage ont des écouteurs
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_1p5()
            // Le bandeau du tiroir est une bande permanente : le budget
            // vertical le compte quand il répartit la fenêtre.
            .h(px(crate::ui_metrics::UiMetrics::DRAWER_HEADER_HEIGHT))
            .bg(theme.bg_surface)
            .border_t_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child("CALIX OPERATION LOG"),
                    )
                    // La commande du produit, puis ce que le moteur a
                    // réellement lancé. Les deux ne sont jamais la même
                    // chaîne : le produit demande, le moteur obéit.
                    .children(props.product_command.map(|command| {
                        div()
                            .text_xs()
                            .font_family("monospace")
                            .text_color(theme.accent)
                            .child(format!("$ {}", command))
                    }))
                    .child(status_badge),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(raw_toggle_btn)
                    .child(copy_btn)
                    .child(clear_btn)
                    .child(autoscroll_btn)
                    .child(toggle_btn),
            );

        // Le tiroir est une bande qui cède : sa hauteur est son bandeau et le
        // corps demandé, son plafond une part de la colonne, et il rétrécit
        // avant la station (qui a un plancher). Le flex décide — personne ne
        // lui pousse de budget.
        let mut container = div()
            .flex()
            .flex_col()
            .flex_shrink(1.0)
            .min_h(px(crate::ui_metrics::UiMetrics::DRAWER_HEADER_HEIGHT))
            .overflow_hidden()
            .when(!props.overlays, |drawer| {
                drawer.max_h(relative(crate::ui_metrics::UiMetrics::DRAWER_MAX_RATIO))
            })
            .bg(theme.bg_sidebar)
            .child(header.flex_none());

        if props.height > 10.0 {
            let mut body_container = div()
                .h(px(props.height))
                .flex_shrink(1.0)
                .min_h(px(0.0))
                .overflow_hidden()
                .flex()
                .flex_col()
                .w_full();

            if !props.backend_phases.is_empty() {
                let mut phases_strip = div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_1p5()
                    .bg(theme.bg_surface)
                    .border_b_1()
                    .border_color(theme.border);

                for phase in props.backend_phases {
                    let (bg_color, text_color, badge_text) = match &phase.status {
                        BackendPhaseStatus::Pending => {
                            (theme.border, theme.text_muted, "○ Queued".to_string())
                        }
                        BackendPhaseStatus::Running(pct) => {
                            let p_str = if let Some(p) = pct {
                                format!("⟳ {}%", p)
                            } else {
                                "⟳ In progress".to_string()
                            };
                            (theme.warning, theme.bg_app, p_str)
                        }
                        BackendPhaseStatus::Success(s) => {
                            (theme.success, theme.bg_app, format!("✓ {}", s))
                        }
                        BackendPhaseStatus::Failed(e) => {
                            (theme.danger, theme.bg_app, format!("✕ {}", e))
                        }
                    };
                    let card = div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2()
                        .py_0p5()
                        .rounded_sm()
                        .bg(theme.bg_surface_hover)
                        .border_1()
                        .border_color(theme.border)
                        .text_xs()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(phase.name.clone()),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_xs()
                                .bg(bg_color)
                                .text_color(text_color)
                                .font_weight(FontWeight::MEDIUM)
                                .child(badge_text),
                        );
                    phases_strip = phases_strip.child(card);
                }
                body_container = body_container.child(phases_strip);
            }

            if auto_scroll && !props.logs.is_empty() {
                props
                    .scroll_handle
                    .scroll_to_item(props.logs.len().saturating_sub(1));
            }

            let mut log_content = div()
                .id("log_drawer_scroll")
                .flex()
                .flex_1()
                .flex_col()
                .overflow_scroll()
                .track_scroll(props.scroll_handle)
                .p_3()
                .bg(theme.bg_sidebar)
                .text_xs();

            if props.logs.is_empty() {
                log_content = log_content.child(
                    div()
                        .text_color(theme.text_muted)
                        .child("No operation logs yet. Calix stdout and stderr execution output will appear here."),
                );
            } else {
                for (idx, entry) in props.logs.iter().enumerate() {
                    let color = if entry.is_stderr {
                        theme.danger
                    } else if entry.text.starts_with(">>>") {
                        theme.accent
                    } else {
                        theme.text_primary
                    };

                    let display_text = if props.show_raw_logs {
                        entry.raw.as_deref().unwrap_or(entry.text.as_str())
                    } else {
                        entry.text.as_str()
                    };

                    log_content = log_content.child(
                        div()
                            .id(idx)
                            .py_0p5()
                            .font_family("monospace")
                            .text_color(color)
                            .child(display_text.to_string()),
                    );
                }
            }

            body_container = body_container.child(log_content);
            container = container.child(body_container);
        }

        container
    }
}

/// Supprime les séquences d'échappement ANSI pour garantir un rendu textuel propre
pub fn sanitize_ansi(s: &str) -> String {
    let mut clean = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            clean.push(c);
        }
    }
    clean
}
