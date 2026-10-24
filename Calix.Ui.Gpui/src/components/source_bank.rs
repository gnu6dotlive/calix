use crate::state::motion::{waterfall_duration, waterfall_entry, Weight, WaterfallStep};
use crate::state::package_store::{SourceHealthMap, SourceHealthStatus};
use crate::state::sources::SourceAxisEntry;
use crate::state::PackageSourceKind;
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;
use std::time::Duration;

/// Ce que fait une ouverture de la banque : elle bascule une source du
/// périmètre, ou relance une source en échec.
pub type SourceToggleHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;
pub type SourceRetryHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;

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

/// The name the proof measures the aperture under, one per source.
fn aperture_selector(kind: PackageSourceKind) -> &'static str {
    match kind {
        PackageSourceKind::Alpm => "aperture-alpm",
        PackageSourceKind::Aur => "aperture-aur",
        PackageSourceKind::Flatpak => "aperture-flatpak",
        PackageSourceKind::AppImage => "aperture-appimage",
        PackageSourceKind::Cargo => "aperture-cargo",
        PackageSourceKind::Npm => "aperture-npm",
        PackageSourceKind::Release => "aperture-release",
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
        // The first segment has no rule before it.
        first: bool,
        props: &SourceBankProps,
        on_toggle: SourceToggleHandler,
        step: Option<(WaterfallStep, Duration)>,
    ) -> AnyElement {
        let theme = props.theme;
        let on_retry = props.on_retry.clone();
        let kind = entry.kind;
        // A source in the scope is a raised segment, in ink; one left out
        // lies flat in the aluminium, muted.
        let in_scope = entry.selected;
        let failed = health.map(|(_, failed)| failed).unwrap_or(false);
        let name_color = if in_scope {
            theme.text_primary
        } else {
            theme.text_muted
        };
        let count_color = if entry.count == 0 {
            theme.text_muted
        } else {
            theme.text_secondary
        };
        // What it holds is said when asked: a recipe and a binary are not the
        // same thing, and the tooltip says so without a line of its own.
        let says: SharedString = match (entry.inventory_only, health) {
            (true, _) => format!("{} — listed, not queried", entry.holds).into(),
            (false, Some((state, _))) => format!("{} — {}", entry.holds, state.to_lowercase()).into(),
            (false, None) => entry.holds.into(),
        };

        let aperture = div()
            .id(aperture_id(kind))
            // The handle the proof measures: the harness reads the bounds of
            // the aperture under this name. A noop outside the tests.
            .debug_selector(move || aperture_selector(kind).to_string())
            .flex()
            .flex_none()
            .items_center()
            .gap(px(7.0))
            .h(px(28.0))
            .px(px(11.0))
            .when(!first, |el| el.border_l_1().border_color(theme.border))
            .cursor_pointer()
            .hover(move |s| s.bg(theme.bg_surface_hover))
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.bg(theme.bg_surface_hover))
            .tooltip(move |_, cx| crate::components::tooltip::tip(Some(says.clone()), cx))
            .on_mouse_down(MouseButton::Left, {
                let on_toggle = on_toggle.clone();
                move |_ev, window, cx| {
                    on_toggle(kind, window, cx);
                }
            })
            .on_click({
                let on_toggle = on_toggle.clone();
                move |event, window, cx| {
                    if event.is_keyboard() {
                        on_toggle(kind, window, cx);
                    }
                }
            })
            // The source's lamp: lit in the accent while the source is in
            // the scope, a ring when it is left out — an instrument's
            // indicator, read before the name.
            .child(
                div()
                    .size(px(7.0))
                    .rounded_full()
                    .when(in_scope, |lamp| lamp.bg(theme.accent))
                    .when(!in_scope, |lamp| lamp.border_1().border_color(theme.text_muted)),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::MEDIUM)
                    .whitespace_nowrap()
                    .text_color(name_color)
                    .child(entry.kind.short_label()),
            )
            // The count of the scope's rows in this source. A value that
            // changes re-reads and flashes once: it does not glide into place.
            .child({
                let count = div()
                    .id(count_id(kind))
                    .text_xs()
                    .font_features(crate::fonts::figures())
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
            // A source that did not answer: a red mark, and the way out.
            .when(failed, |element| {
                let retry = on_retry.clone();
                element
                    .child(div().size(px(6.0)).rounded_full().bg(theme.danger))
                    .child(
                        div()
                            .id(retry_id(kind))
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.danger)
                            .cursor_pointer()
                            .hover(|s| s.underline())
                            .child("Retry")
                            .on_click(move |_ev, window, cx| {
                                cx.stop_propagation();
                                retry(kind, window, cx);
                            }),
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
                index == 0,
                &props,
                props.on_toggle.clone(),
                steps.get(index).copied().map(|step| (step, total)),
            ));
        }

        div()
            .id("source_bank")
            // The handle the audit measures: the harness reads the bounds of
            // the band under this name. A noop outside the tests.
            .debug_selector(|| "source_bank".to_string())
            .flex()
            .flex_row()
            .items_center()
            .flex_none()
            .w_full()
            .child(
                // One control, its segments side by side: the sources are an
                // axis, not a pile of chips.
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .rounded_md()
                    .border_1()
                    .border_color(props.theme.border)
                    .bg(props.theme.bg_surface)
                    .shadow_xs()
                    .overflow_hidden()
                    .children(apertures),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{aperture_selector, SourceBank, SourceBankProps};
    use crate::state::package_store::SourceHealthMap;
    use crate::state::query::SourceScope;
    use crate::state::sources::{source_axis, SourceAxisEntry};
    use crate::state::{NavScope, PackageSourceKind};
    use crate::theme::Theme;
    use gpui::prelude::*;
    use gpui::{div, point, px, size, Context, Render, Window};
    use std::rc::Rc;

    /// L'arrivée de la banque, mesurée sur l'image : les sept ouvertures
    /// viennent par en dessous, et l'ancre voyage plus loin que les autres.
    ///
    /// La première image est prise dans l'image même où l'animation naît —
    /// l'état de l'animation est créé au premier placement, donc le premier
    /// écart vaut zéro — et la seconde après la vague.
    #[gpui::test]
    fn the_bank_comes_in_from_below_and_lands(cx: &mut gpui::TestAppContext) {
        struct Fixture {
            theme: Theme,
            axis: Vec<SourceAxisEntry>,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                let health = SourceHealthMap::default();
                div().size_full().child(SourceBank::render(SourceBankProps {
                    axis: &self.axis,
                    source_health: &health,
                    theme: &theme,
                    epoch: 4,
                    reduce_motion: false,
                    on_toggle: Rc::new(|_, _, _| {}),
                    on_retry: Rc::new(|_, _, _| {}),
                }))
            }
        }

        let cx = cx.add_empty_window();
        // An axis with rows: the first source is the anchor, the others are
        // lighter, and the empty ones are the lightest of all.
        let axis = source_axis(NavScope::Installed, &[], &SourceScope::all());
        let anchor = crate::state::anchor_index(&axis);
        let selector = aperture_selector(PackageSourceKind::ALL[anchor]);
        let fixture = cx.new(|_| Fixture {
            theme: Theme::dark(),
            axis,
        });

        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(1080.0), px(200.0)),
                |_window, _cx| fixture.clone().into_element(),
            );
        };

        draw(cx);
        let departure = cx
            .debug_bounds(selector)
            .expect("la banque est dessinée");

        // La vague dure un peu plus de 400 ms ; on attend plus, et on dessine
        // encore.
        std::thread::sleep(std::time::Duration::from_millis(700));
        draw(cx);
        let arrival = cx
            .debug_bounds(selector)
            .expect("la banque est dessinée");

        // L'ancre est venue de plus bas que les autres : elle porte le
        // plus de lignes, et son voyage est le plus long.
        assert_eq!(
            departure.origin.y - arrival.origin.y,
            px(14.0),
            "l'ancre est arrivée quatorze pixels plus bas, et elle est montée à sa place"
        );
    }

    /// La porte de l'interaction : un clic sur une ouverture de la banque
    /// bascule cette source.
    ///
    /// La banque est l'axe des sources : c'est par elle que l'on change ce
    /// que l'on cherche. Un clic qui ne bascule rien est une banque morte —
    /// et la porte le dit.
    #[gpui::test]
    fn a_click_on_an_aperture_toggles_that_source(cx: &mut gpui::TestAppContext) {
        use gpui::prelude::*;
        use gpui::{div, point, px, size, Context, Render, Window};
        use std::cell::RefCell;
        use std::rc::Rc;

        struct Fixture {
            theme: Theme,
            axis: Vec<SourceAxisEntry>,
            toggled: Rc<RefCell<Vec<PackageSourceKind>>>,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                let toggled = self.toggled.clone();
                div().size_full().child(SourceBank::render(SourceBankProps {
                    axis: &self.axis,
                    source_health: &crate::state::package_store::SourceHealthMap::default(),
                    theme: &theme,
                    epoch: 1,
                    reduce_motion: true,
                    on_toggle: Rc::new(move |kind, _w, _cx| {
                        toggled.borrow_mut().push(kind);
                    }),
                    on_retry: Rc::new(|_, _, _| {}),
                }))
            }
        }

        let cx = cx.add_empty_window();
        let toggled = Rc::new(RefCell::new(Vec::new()));
        let axis = crate::state::sources::source_axis(
            crate::state::NavScope::Installed,
            &[],
            &crate::state::query::SourceScope::all(),
        );
        let fixture = cx.new(|_| Fixture {
            theme: Theme::dark(),
            axis: axis.clone(),
            toggled: toggled.clone(),
        });

        for entry in &axis {
            toggled.borrow_mut().clear();
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(1080.0), px(200.0)),
                |_window, _cx| fixture.clone().into_element(),
            );
            let bounds = cx
                .debug_bounds(aperture_selector(entry.kind))
                .unwrap_or_else(|| panic!("the aperture is drawn"));
            cx.simulate_click(
                point(
                    bounds.origin.x + bounds.size.width / 2.0,
                    bounds.origin.y + bounds.size.height / 2.0,
                ),
                gpui::Modifiers::default(),
            );
            assert_eq!(
                *toggled.borrow(),
                vec![entry.kind],
                "le clic sur l'ouverture de {:?} l'a basculée",
                entry.kind
            );
        }
    }
}
