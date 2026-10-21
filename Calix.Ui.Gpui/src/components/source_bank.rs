use crate::components::query_workbench::{SourceRetryHandler, SourceToggleHandler};
use crate::state::motion::{waterfall_duration, waterfall_entry, Weight, WaterfallStep};
use crate::state::package_store::{SourceHealthMap, SourceHealthStatus};
use crate::state::sources::SourceAxisEntry;
use crate::state::PackageSourceKind;
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::time::Duration;

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
    /// The arrival: the bank comes in as a wave, not as a queue.
    pub epoch: usize,
    /// The user asked for less movement.
    pub reduce_motion: bool,
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

/// The id of the count, one per source.
fn count_id(kind: PackageSourceKind) -> &'static str {
    match kind {
        PackageSourceKind::Alpm => "source_count_alpm",
        PackageSourceKind::Aur => "source_count_aur",
        PackageSourceKind::Flatpak => "source_count_flatpak",
        PackageSourceKind::AppImage => "source_count_appimage",
        PackageSourceKind::Cargo => "source_count_cargo",
        PackageSourceKind::Npm => "source_count_npm",
        PackageSourceKind::Release => "source_count_release",
    }
}

/// The id of the arrival animation, one per source.
fn arrival_id(kind: PackageSourceKind) -> &'static str {
    match kind {
        PackageSourceKind::Alpm => "source_arrival_alpm",
        PackageSourceKind::Aur => "source_arrival_aur",
        PackageSourceKind::Flatpak => "source_arrival_flatpak",
        PackageSourceKind::AppImage => "source_arrival_appimage",
        PackageSourceKind::Cargo => "source_arrival_cargo",
        PackageSourceKind::Npm => "source_arrival_npm",
        PackageSourceKind::Release => "source_arrival_release",
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
        step: Option<(WaterfallStep, Duration)>,
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

        let aperture = div()
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
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_color(if selected {
                        theme.accent
                    } else {
                        theme.text_primary
                    })
                    .child(entry.label),
            )
            // What it holds: the non-interchangeability, stated up front. The
            // line is cut rather than wrapped: the bank is a band, and a band
            // does not grow.
            .child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .child(entry.holds),
            )
            // The count of the scope's rows in this source. A value that
            // changes re-reads and flashes once: it does not glide into place.
            .child({
                let count = div()
                    .id(count_id(kind))
                    .text_sm()
                    .font_family("monospace")
                    .text_color(count_color)
                    .child(entry.count.to_string());
                if props.reduce_motion {
                    count.into_any_element()
                } else {
                    count
                        .with_animation(
                            (count_id(kind), entry.count),
                            Animation::new(crate::state::MotionDurations::EMPHASIS),
                            |element, delta| {
                                element
                                    .opacity(1.0 - 0.45 * (std::f32::consts::PI * delta).sin())
                            },
                        )
                        .into_any_element()
                }
            })
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
            });

        // L'arrivée : la banque arrive par en dessous, dans une vague qui
        // s'accélère, et elle s'allume d'un coup — une apparition en fondu se
        // lirait comme un fondu, pas comme une arrivée.
        match step {
            Some((step, total)) if !props.reduce_motion => {
                let total_ms = total.as_millis().max(1) as f32;
                let delay = step.delay.as_millis() as f32 / total_ms;
                let travel = step.travel;
                aperture
                    .with_animation(
                        (arrival_id(kind), props.epoch),
                        Animation::new(total).with_easing(crate::state::motion::ease_out),
                        move |element, delta| {
                            let local = ((delta - delay) / (1.0 - delay)).clamp(0.0, 1.0);
                            element
                                .top(px(travel * (1.0 - crate::state::motion::ease_out(local))))
                                .opacity(if local > 0.0 { 1.0 } else { 0.0 })
                        },
                    )
                    .into_any_element()
            }
            _ => aperture.into_any_element(),
        }
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

        // Les poids : l'ouverture retenue est l'ancre, elle voyage plus loin ;
        // une source sans ligne est légère, elle s'ajuste.
        let weights: Vec<Weight> = props
            .axis
            .iter()
            .map(|entry| {
                if entry.selected {
                    Weight::Heavy
                } else if entry.count == 0 {
                    Weight::Light
                } else {
                    Weight::Normal
                }
            })
            .collect();
        let steps = waterfall_entry(&weights);
        let total = waterfall_duration(&steps);

        let mut apertures = Vec::with_capacity(props.axis.len());
        for (index, entry) in props.axis.iter().enumerate() {
            apertures.push(Self::render_aperture(
                entry,
                health_of(entry.kind),
                &props,
                props.on_toggle.clone(),
                steps.get(index).copied().map(|step| (step, total)),
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
