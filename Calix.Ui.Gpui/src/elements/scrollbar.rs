//! Une barre de défilement pour tout ce qui défile.
//!
//! Adapté de Skwir (`crates/ui/src/elements/scrollbar.rs`, Apache-2.0) : chez
//! Skwir, toute vue qui défile a sa barre ; Calix n'en avait aucune, et rien
//! ne disait combien de catalogue restait sous le pli.
//!
//! La barre se pose sur le bord droit de la zone au lieu d'y prendre une
//! colonne : l'ajouter ne décale jamais la vue en dessous. Comme les autres
//! éléments d'ici, elle lit les tailles là où elles sont réelles : le pouce est
//! placé dans `prepaint` à partir des nombres de la poignée de défilement,
//! jamais d'une taille mesurée pendant un `render`.
//!
//! La vue lui donne la poignée avec laquelle elle défile déjà : un
//! `ScrollHandle` (`overflow_y_scroll` + `track_scroll`), ou celui qui porte
//! une `uniform_list` (`UniformListScrollHandle`, voir `for_list`).

use crate::theme::Theme;
use gpui::{
    px, relative, Along, App, Axis, Bounds, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, ScrollHandle, Style, UniformListScrollHandle, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// La largeur de la gouttière où vit la barre.
const GUTTER: f32 = 12.;
/// La barre au repos, et sous le pointeur.
const BAR: f32 = 5.;
const BAR_HOVER: f32 = 9.;
/// Quelle que soit la longueur du contenu, le pouce reste saisissable.
const MIN_THUMB: f32 = 24.;

/// La position de défilement d'une vue, en pixels depuis son début.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    offset: f32,
    /// Jusqu'où elle peut défiler ; zéro quand tout tient.
    max: f32,
    viewport: f32,
}

impl Metrics {
    fn of(handle: &ScrollHandle) -> Self {
        Metrics {
            offset: -f32::from(handle.offset().along(Axis::Vertical)),
            max: f32::from(handle.max_offset().along(Axis::Vertical)),
            viewport: f32::from(handle.bounds().size.along(Axis::Vertical)),
        }
    }

    /// S'il y a quelque chose à faire défiler.
    fn scrolls(self) -> bool {
        self.max > 1. && self.viewport > 0.
    }
}

/// Où le pointeur a saisi le pouce, gardé d'une image à l'autre.
#[derive(Clone, Default)]
struct Grab(Rc<Cell<Option<f32>>>);

pub struct Scrollbar {
    id: ElementId,
    handle: ScrollHandle,
}

/// Une barre verticale sur `handle`. L'id doit rester le même d'une image à
/// l'autre, sinon un glissement s'arrête dès que la vue se redessine.
pub fn vertical(id: impl Into<ElementId>, handle: &ScrollHandle) -> Scrollbar {
    Scrollbar {
        id: id.into(),
        handle: handle.clone(),
    }
}

/// La barre d'une `uniform_list` : elle défile par la poignée de base de la
/// liste, la même que la liste lit.
pub fn for_list(id: impl Into<ElementId>, list: &UniformListScrollHandle) -> Scrollbar {
    let handle = list.0.borrow().base_handle.clone();
    Scrollbar {
        id: id.into(),
        handle,
    }
}

impl IntoElement for Scrollbar {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

/// Le pouce dans `track`, ou `None` quand tout tient.
fn thumb(track: Bounds<Pixels>, metrics: Metrics) -> Option<Bounds<Pixels>> {
    let axis = Axis::Vertical;
    let length = f32::from(track.size.along(axis));
    if !metrics.scrolls() || length <= MIN_THUMB {
        return None;
    }
    let content = metrics.viewport + metrics.max;
    let thumb = (length * metrics.viewport / content).clamp(MIN_THUMB, length);
    let travel = length - thumb;
    let progress = (metrics.offset / metrics.max).clamp(0., 1.);
    Some(Bounds::new(
        track
            .origin
            .apply_along(axis, |start| start + px(travel * progress)),
        track.size.apply_along(axis, |_| px(thumb)),
    ))
}

impl Element for Scrollbar {
    type RequestLayoutState = ();
    type PrepaintState = (Hitbox, Option<Bounds<Pixels>>, Metrics);

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut style = Style {
            position: gpui::Position::Absolute,
            ..Style::default()
        };
        style.inset.top = px(0.).into();
        style.inset.right = px(0.).into();
        style.size.width = px(GUTTER).into();
        style.size.height = relative(1.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        (): &mut (),
        window: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        let metrics = Metrics::of(&self.handle);
        let thumb = thumb(bounds, metrics);
        // La gouttière ne prend le pointeur que tant qu'elle porte une barre :
        // un appui ici fait défiler, il ne sélectionne pas la rangée dessous.
        // Sans rien à faire défiler, elle n'est pas là.
        let behavior = if thumb.is_some() {
            HitboxBehavior::BlockMouse
        } else {
            HitboxBehavior::Normal
        };
        (window.insert_hitbox(bounds, behavior), thumb, metrics)
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        (): &mut (),
        (hitbox, thumb, metrics): &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (Some(thumb), Some(global_id)) = (*thumb, global_id) else {
            return;
        };
        let bounds = hitbox.bounds;
        let metrics = *metrics;
        let handle = self.handle.clone();
        let axis = Axis::Vertical;
        // En travers : l'épaisseur dessinée, contre le bord, pour que la barre
        // épouse le bord de la zone.
        let across = axis.invert();
        let hug = move |area: Bounds<Pixels>, thickness: Pixels| {
            Bounds::new(
                area.origin.apply_along(across, |start| {
                    start + bounds.size.along(across) - thickness
                }),
                area.size.apply_along(across, |_| thickness),
            )
        };
        window.with_element_state::<Grab, _>(global_id, |grab, window| {
            let grab: Grab = grab.unwrap_or_default();
            let held = grab.0.get().is_some();
            let hovered = hitbox.is_hovered(window);
            let theme = Theme::of(cx);
            let active = held || hovered;
            let width = px(if active { BAR_HOVER } else { BAR });
            if active {
                window.paint_quad(gpui::fill(hug(bounds, width), theme.bg_surface_hover));
            }
            let ink: Hsla = theme.text_muted.into();
            window.paint_quad(gpui::quad(
                hug(thumb, width),
                px(f32::from(width) / 2.),
                ink.opacity(if active { 0.8 } else { 0.45 }),
                px(0.),
                gpui::transparent_black(),
                gpui::BorderStyle::default(),
            ));

            let travel = f32::from(bounds.size.along(axis) - thumb.size.along(axis));
            let start = bounds.origin.along(axis);
            let to_offset = move |at: gpui::Point<Pixels>, grabbed: f32| {
                if travel <= 0. {
                    return 0.;
                }
                let position = f32::from(at.along(axis) - start) - grabbed;
                (position / travel).clamp(0., 1.) * metrics.max
            };
            let scroll_to = |handle: &ScrollHandle, to: f32| {
                handle.set_offset(handle.offset().apply_along(Axis::Vertical, |_| px(-to)));
            };

            let down = grab.clone();
            let hit = hitbox.clone();
            let on_down = handle.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, _| {
                if !phase.bubble() || event.button != MouseButton::Left || !hit.is_hovered(window)
                {
                    return;
                }
                // Sur le pouce : glisser depuis où il a été saisi. À côté :
                // y sauter, puis glisser depuis son milieu.
                let grabbed = if thumb.contains(&event.position) {
                    f32::from(event.position.along(axis) - thumb.origin.along(axis))
                } else {
                    let middle = f32::from(thumb.size.along(axis)) / 2.;
                    scroll_to(&on_down, to_offset(event.position, middle));
                    middle
                };
                down.0.set(Some(grabbed));
                window.refresh();
            });

            let moved = grab.clone();
            let on_move = handle.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, _| {
                if !phase.bubble() {
                    return;
                }
                let Some(grabbed) = moved.0.get() else {
                    return;
                };
                // Un relâchement hors de la fenêtre n'arrive jamais ici.
                if event.pressed_button != Some(MouseButton::Left) {
                    moved.0.set(None);
                    return;
                }
                scroll_to(&on_move, to_offset(event.position, grabbed));
                window.refresh();
            });

            let up = grab.clone();
            window.on_mouse_event(move |event: &MouseUpEvent, phase, window, _| {
                if phase.bubble() && event.button == MouseButton::Left && up.0.take().is_some() {
                    window.refresh();
                }
            });
            ((), grab)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{thumb, vertical, Metrics, MIN_THUMB};
    use core::prelude::v1::test;
    use gpui::prelude::*;
    use gpui::{div, point, px, size, Bounds, Modifiers, MouseButton, ScrollHandle, Window};

    /// A list of a hundred rows of 20 px in a 200 px window, with its bar.
    struct Long {
        scroll: ScrollHandle,
    }

    impl Render for Long {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div()
                .w(px(300.))
                .h(px(200.))
                .relative()
                .child(
                    div()
                        .id("long")
                        .size_full()
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll)
                        .children((0..100).map(|i| div().h(px(20.)).child(format!("row {i}")))),
                )
                .child(vertical("long_bar", &self.scroll))
        }
    }

    #[gpui::test]
    fn a_press_in_the_gutter_scrolls_there_and_a_drag_follows(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| crate::theme::Theme::install(crate::theme::Theme::dark(), cx));
        let scroll = ScrollHandle::new();
        let (_view, cx) = cx.add_window_view({
            let scroll = scroll.clone();
            move |_, _| Long { scroll }
        });
        cx.run_until_parked();
        assert_eq!(scroll.max_offset().y, px(1800.), "the list measured itself");
        // A press at the bottom of the gutter goes to the end.
        let bottom = point(px(295.), px(199.));
        cx.simulate_mouse_down(bottom, MouseButton::Left, Modifiers::none());
        assert_eq!(scroll.offset().y, -px(1800.));
        // Dragged back to the top, the list follows the pointer.
        cx.simulate_mouse_move(point(px(295.), px(0.)), MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_up(point(px(295.), px(0.)), MouseButton::Left, Modifiers::none());
        assert_eq!(scroll.offset().y, px(0.));
    }

    fn track() -> Bounds<gpui::Pixels> {
        Bounds::new(point(px(0.), px(0.)), size(px(10.), px(200.)))
    }

    #[test]
    fn the_thumb_is_as_long_as_the_view_is_of_the_content() {
        // La moitié du contenu se voit : la moitié de la piste, en haut.
        let half = thumb(
            track(),
            Metrics {
                offset: 0.,
                max: 200.,
                viewport: 200.,
            },
        )
        .expect("a thumb");
        assert_eq!(half.size.height, px(100.));
        assert_eq!(half.origin.y, px(0.));
        // Défilée jusqu'au bout, elle touche le bas de la piste.
        let end = thumb(
            track(),
            Metrics {
                offset: 200.,
                max: 200.,
                viewport: 200.,
            },
        )
        .expect("a thumb");
        assert_eq!(end.origin.y + end.size.height, px(200.));
    }

    #[test]
    fn nothing_to_scroll_means_no_thumb() {
        assert!(thumb(
            track(),
            Metrics {
                offset: 0.,
                max: 0.,
                viewport: 500.,
            },
        )
        .is_none());
    }

    #[test]
    fn a_very_long_catalogue_keeps_a_grabbable_thumb() {
        // Des dizaines de milliers de paquets : le pouce reste saisissable.
        let long = thumb(
            track(),
            Metrics {
                offset: 500_000.,
                max: 1_000_000.,
                viewport: 200.,
            },
        )
        .expect("a thumb");
        assert_eq!(long.size.height, px(MIN_THUMB));
        assert_eq!(long.origin.y, px((200. - MIN_THUMB) / 2.));
    }
}
