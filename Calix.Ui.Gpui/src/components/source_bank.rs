use crate::components::query_workbench::{SourceRetryHandler, SourceToggleHandler};
use crate::state::package_store::{SourceHealthMap, SourceHealthStatus};
use crate::state::sources::SourceAxisEntry;
use crate::state::PackageSourceKind;
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;

/// The source bank: seven apertures, one per source.
///
/// The sources are an axis, not a filter. A bank is a control, not a place:
/// it is always present, under the header, and every aperture says what the
/// source holds and how many rows of the current scope it carries. A recipe
/// and a binary are not the same thing, and the bank says so before the user
/// asks.
pub struct SourceBankProps<'a> {
    pub axis: &'a [SourceAxisEntry],
    pub source_health: &'a SourceHealthMap,
    pub theme: &'a Theme,
    pub on_toggle: SourceToggleHandler,
    pub on_retry: SourceRetryHandler,
}

/// The element ids of the bank, one per source. They must outlive the frame.
fn aperture_id(kind: PackageSourceKind) -> &'static str {
    match kind {
        PackageSourceKind::Alpm => "source_aperture_alpm",
        PackageSourceKind::Aur => "source_aperture_aur",
        PackageSourceKind::Flatpak => "source_aperture_flatpak",
        PackageSourceKind::AppImage => "source_aperture_appimage",
        PackageSourceKind::Cargo => "source_aperture_cargo",
        PackageSourceKind::Npm => "source_aperture_npm",
        PackageSourceKind::Release => "source_aperture_release",
    }
}

fn retry_id(kind: PackageSourceKind) -> &'static str {
    match kind {
        PackageSourceKind::Alpm => "source_retry_alpm",
        PackageSourceKind::Aur => "source_retry_aur",
        PackageSourceKind::Flatpak => "source_retry_flatpak",
        PackageSourceKind::AppImage => "source_retry_appimage",
        PackageSourceKind::Cargo => "source_retry_cargo",
        PackageSourceKind::Npm => "source_retry_npm",
        PackageSourceKind::Release => "source_retry_release",
    }
}

pub struct SourceBank;

impl SourceBank {
    /// One aperture: the name, the kind, the count, the state.
    fn render_aperture(
        entry: &SourceAxisEntry,
        health: Option<(&'static str, bool)>,
        props: &SourceBankProps,
        on_toggle: SourceToggleHandler,
    ) -> AnyElement {
        let theme = props.theme;
        let on_retry = props.on_retry.clone();
        let kind = entry.kind;
        let selected = entry.selected;
        let border = if selected {
            theme.accent
        } else {
            theme.border
        };
        let count_color = if entry.count == 0 {
            theme.text_muted
        } else {
            theme.text_primary
        };

        let health_line = health.map(|(label, failed)| {
            div()
                .text_xs()
                .text_color(if failed { theme.danger } else { theme.text_muted })
                .child(label)
        });

        div()
            .id(aperture_id(kind))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .min_w(px(96.0))
            .flex_1()
            .px_3()
            .py_2()
            .border_b_2()
            .border_color(border)
            .bg(if selected {
                theme.bg_surface_hover
            } else {
                theme.bg_surface
            })
            .cursor_pointer()
            .hover(move |s| s.bg(theme.bg_surface_hover))
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_b_2().border_color(theme.accent))
            .on_mouse_down(MouseButton::Left, {
                let on_toggle = on_toggle.clone();
                move |_ev, window, cx| {
                    on_toggle(kind, window, cx);
                }
            })
            .on_key_down({
                let on_toggle = on_toggle.clone();
                move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        on_toggle(kind, window, cx);
                    }
                }
            })
            // The name of the source.
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(if selected {
                        theme.accent
                    } else {
                        theme.text_primary
                    })
                    .child(entry.label),
            )
            // What it holds: the non-interchangeability, stated up front.
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(entry.holds),
            )
            // The count of the scope's rows in this source.
            .child(
                div()
                    .text_sm()
                    .font_family("monospace")
                    .text_color(count_color)
                    .child(entry.count.to_string()),
            )
            // The state of the source: a source that did not answer says so.
            .children(health_line)
            .when(
                health.map(|(_, failed)| failed).unwrap_or(false),
                |element| {
                    let retry = on_retry.clone();
                    element.child(
                        div()
                            .id(retry_id(kind))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.danger)
                            .cursor_pointer()
                            .child("Retry")
                            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                                retry(kind, window, cx);
                            }),
                    )
                },
            )
            // The inventory ecosystems are listed, never queried.
            .when(entry.inventory_only, |element| {
                element.child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("listed, not queried"),
                )
            })
            .into_any_element()
    }

    pub fn render(props: SourceBankProps) -> impl IntoElement {
        let theme = props.theme;
        let health_of = |kind| {
            props.source_health.get(&kind).and_then(|health| match health.status {
                SourceHealthStatus::Failed => Some(("Failed", true)),
                SourceHealthStatus::Disabled => Some(("Disabled", false)),
                SourceHealthStatus::Loading => Some(("Loading...", false)),
                // A source that answered says nothing.
                SourceHealthStatus::Ready => None,
            })
        };

        let mut apertures = Vec::with_capacity(props.axis.len());
        for entry in props.axis {
            apertures.push(Self::render_aperture(
                entry,
                health_of(entry.kind),
                &props,
                props.on_toggle.clone(),
            ));
        }

        div()
            .id("source_bank")
            .flex()
            .flex_row()
            .gap(px(6.0))
            .w_full()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.bg_app)
            .children(apertures)
    }
}
