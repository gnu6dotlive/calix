use crate::state::motion::{
    carrier_travel, ease_in_out, MotionDurations, SelectionMarker,
};
use crate::state::session::PackageViewMode;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type ViewModeHandler = Rc<dyn Fn(PackageViewMode, &mut Window, &mut App) + 'static>;

/// La cellule d'un mode, et l'écart entre deux cellules.
///
/// La marque qui voyage a besoin d'une géométrie fixe : c'est ce qui permet
/// de la poser sans mesurer, et de la faire glisser d'une cellule à l'autre
/// sans jamais consulter la mise en page.
pub const VIEW_MODE_CELL: f32 = 26.0;
pub const VIEW_MODE_GAP: f32 = 2.0;
/// La distance entre deux cellules : la largeur d'une cellule et l'écart.
pub const VIEW_MODE_STRIDE: f32 = VIEW_MODE_CELL + VIEW_MODE_GAP;

/// Où se pose la marque du mode : `index` cellules plus loin.
pub fn thumb_left(index: usize) -> f32 {
    index as f32 * VIEW_MODE_STRIDE
}

/// Où passe la marque pendant le voyage, en pixels.
///
/// Le porteur est le même que celui de la liste et du rail : il part de la
/// cellule quittée et se pose sur la cellule choisie, sans sauter.
pub fn thumb_travel(from: usize, to: usize, delta: f32) -> f32 {
    carrier_travel(from as f32, to as f32, delta) * VIEW_MODE_STRIDE
}

pub struct ViewModeSwitcherProps<'a> {
    pub current_mode: PackageViewMode,
    pub theme: &'a Theme,
    pub reduce_motion: bool,
    /// Le porteur : la marque qui voyage de la cellule quittée à la cellule
    /// choisie. Le commutateur ne le possède pas — il le dessine.
    pub marker: Option<SelectionMarker>,
    pub on_select_mode: ViewModeHandler,
}

pub struct ViewModeSwitcher;

impl ViewModeSwitcher {
    pub fn render(props: ViewModeSwitcherProps) -> impl IntoElement {
        let theme = props.theme;
        let modes = [PackageViewMode::Cards, PackageViewMode::Table];

        // La marque lumineuse : le fond de la cellule active est un objet qui
        // voyage, pas une couleur qui change de place. La destination, elle,
        // s'allume d'un coup par la couleur de son icône — le fond dit où la
        // vue est, la marque dit d'où elle vient.
        let index = props
            .marker
            .map(|marker| marker.index)
            .unwrap_or(props.current_mode.index());
        let fill = div()
            .id("view_mode_fill")
            .absolute()
            .top(px(0.0))
            .left(px(thumb_left(index)))
            .size(px(VIEW_MODE_CELL))
            .rounded_sm()
            .bg(theme.bg_surface_active)
            // The handle the proof measures: the harness reads the bounds of
            // the mark under this name. A noop outside the tests.
            .debug_selector(|| "view_mode_fill".to_string());
        let fill: AnyElement = match props.marker {
            Some(marker) if !props.reduce_motion => {
                let from = marker.from.unwrap_or(marker.index);
                let to = marker.index;
                fill.with_animation(
                    ("view_mode_fill", marker.epoch),
                    Animation::new(MotionDurations::EMPHASIS).with_easing(ease_in_out),
                    move |element, delta| element.left(px(thumb_travel(from, to, delta))),
                )
                .into_any_element()
            }
            _ => fill.into_any_element(),
        };

        let mut container = div()
            .id("workbench_view_mode_switcher")
            .relative()
            .flex()
            .items_center()
            .gap(px(VIEW_MODE_GAP))
            // La marque est peinte d'abord : les icônes se posent dessus.
            .child(fill);

        for mode in modes {
            let is_active = props.current_mode == mode;
            let on_select = props.on_select_mode.clone();
            let on_select_key = props.on_select_mode.clone();
            let element_id = match mode {
                PackageViewMode::Cards => "view_mode_cards",
                PackageViewMode::Table => "view_mode_table",
            };
            let focus_border = theme.border_focus;
            let hover_bg = theme.bg_surface_hover;
            container =
                container.child(
                    div()
                        .id(element_id)
                        // The handles the bench measures and clicks: the
                        // harness reads the bounds of a cell under this name.
                        // A noop outside the tests.
                        .debug_selector(|| element_id.to_string())
                        .focusable()
                        .tab_stop(true)
                        .focus(move |s| s.border_1().border_color(focus_border))
                        .on_click(move |event, window, cx| {
                            if event.is_keyboard() {
                                on_select_key(mode, window, cx);
                            }
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(VIEW_MODE_CELL))
                        .rounded_sm()
                        .cursor_pointer()
                        .text_color(if is_active {
                            theme.accent_text
                        } else {
                            theme.text_muted
                        })
                        .hover(move |s| s.bg(hover_bg).text_color(theme.text_primary))
                        .child(svg().path(mode.icon().path()).size(px(14.0)).text_color(
                            if is_active {
                                theme.accent_text
                            } else {
                                theme.text_muted
                            },
                        ))
                        .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                            on_select(mode, window, cx);
                        }),
                );
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::{
        thumb_left, thumb_travel, ViewModeSwitcher, ViewModeSwitcherProps, VIEW_MODE_STRIDE,
    };
    use crate::state::motion::SelectionMarker;
    use crate::state::session::PackageViewMode;
    use crate::theme::Theme;
    use core::prelude::v1::test;

    #[test]
    fn the_fill_lands_on_the_cell_it_marks() {
        // Deux cellules de 26 px séparées de 2 : la marque est à 0 ou à 28.
        assert_eq!(thumb_left(0), 0.0);
        assert_eq!(thumb_left(1), 28.0);

        // Au départ et à l'arrivée elle est exactement sur la cellule.
        assert_eq!(thumb_travel(0, 1, 0.0), 0.0);
        assert!((thumb_travel(0, 1, 1.0) - 28.0).abs() < 1e-4);

        // Entre les deux, elle a quitté la cellule sans l'avoir rejointe.
        let mid = thumb_travel(0, 1, 0.5);
        assert!(mid > 0.0 && mid < 28.0, "la marque voyage : {mid}");

        // Le retour parcourt le même chemin.
        assert!((thumb_travel(0, 1, 0.5) + thumb_travel(1, 0, 0.5) - 28.0).abs() < 1e-4);

        // Une marque sans départ reste immobile.
        assert_eq!(thumb_travel(1, 1, 0.42), 28.0);
    }

    /// Le clic, dans le banc : la porte de l'interaction.
    ///
    /// Depuis v0.2.5, les notes de version disaient que les clics n'étaient
    /// pas prouvés — le pointeur synthétique n'atteignait pas le client
    /// Wayland sur la machine où la preuve avait été prise. Le pointeur
    /// synthétique se battait avec le compositeur : il volait le focus, et il
    /// ne atterrissait jamais.
    ///
    /// Le banc, lui, n'a pas ce problème : il expédie l'événement *dans* la
    /// fenêtre — un vrai test de collision, un vrai ordre de propagation, un
    /// vrai gestionnaire. La preuve n'a pas besoin d'un bureau.
    ///
    /// Ce test est le premier de la série : un clic sur le vrai commutateur
    /// de vue, et le mode qui change.
    #[gpui::test]
    fn a_click_on_the_switcher_changes_the_view(cx: &mut gpui::TestAppContext) {
        use gpui::prelude::*;
        use gpui::{div, point, px, size, Context, Render, Window};
        use std::cell::Cell;
        use std::rc::Rc;

        struct Fixture {
            theme: Theme,
            chosen: Rc<Cell<Option<PackageViewMode>>>,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                let chosen = self.chosen.clone();
                div().size_full().child(ViewModeSwitcher::render(
                    ViewModeSwitcherProps {
                        current_mode: PackageViewMode::Table,
                        theme: &theme,
                        reduce_motion: false,
                        marker: None,
                        on_select_mode: Rc::new(move |mode, _w, _cx| {
                            chosen.set(Some(mode));
                        }),
                    },
                ))
            }
        }

        let cx = cx.add_empty_window();
        let theme = Theme::dark();
        let chosen = Rc::new(Cell::new(None));

        let fixture = cx.new(|_| Fixture {
            theme: theme.clone(),
            chosen: chosen.clone(),
        });

        cx.draw(
            point(px(0.0), px(0.0)),
            size(px(400.0), px(200.0)),
            |_window, _cx| fixture.clone().into_element(),
        );

        // The two cells, measured.
        let cards = cx
            .debug_bounds("view_mode_cards")
            .expect("the cards cell is drawn");
        let table = cx
            .debug_bounds("view_mode_table")
            .expect("the table cell is drawn");

        // The click, on the cards cell: the handler must be reached.
        cx.simulate_click(
            point(
                cards.origin.x + cards.size.width / 2.0,
                cards.origin.y + cards.size.height / 2.0,
            ),
            gpui::Modifiers::default(),
        );

        assert_eq!(
            chosen.get(),
            Some(PackageViewMode::Cards),
            "le clic sur la cellule des cartes a changé le mode"
        );

        // And the other cell, on its own: the two are distinct.
        //
        // The frame between the two clicks is not a formality: a click is a
        // sequence of events separated by frames, exactly as it is for a
        // user. The first click moves the focus; the focus changes the
        // element's style; the window repaints; the *next* click is hit
        // against the hitboxes of that repaint. Without the frame, the
        // second click is dispatched against a stale frame and lands
        // nowhere — which is the bench's protocol, not a defect of the
        // application.
        chosen.set(None);
        let frame = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(400.0), px(200.0)),
                |_window, _cx| fixture.clone().into_element(),
            );
        };
        frame(cx);
        cx.simulate_click(
            point(
                table.origin.x + table.size.width / 2.0,
                table.origin.y + table.size.height / 2.0,
            ),
            gpui::Modifiers::default(),
        );
        assert_eq!(
            chosen.get(),
            Some(PackageViewMode::Table),
            "et le clic sur la cellule du tableau a changé le mode"
        );

        // And the first one again: the two cells are distinct, and both
        // answer.
        chosen.set(None);
        frame(cx);
        cx.simulate_click(
            point(
                cards.origin.x + cards.size.width / 2.0,
                cards.origin.y + cards.size.height / 2.0,
            ),
            gpui::Modifiers::default(),
        );
        assert_eq!(
            chosen.get(),
            Some(PackageViewMode::Cards),
            "et le clic sur la cellule des cartes a changé le mode"
        );
    }

    #[test]
    fn the_two_modes_have_two_cells() {
        // L'index du mode est la cellule : les deux se suivent, et la
        // géométrie du commutateur ne dépend que de cet index.
        assert_eq!(PackageViewMode::Cards.index(), 0);
        assert_eq!(PackageViewMode::Table.index(), 1);
        assert_eq!(thumb_left(PackageViewMode::Table.index()), VIEW_MODE_STRIDE);
    }

    /// Le voyage de la marque, mesuré sur l'image : elle part de la cellule
    /// qu'elle quittait et se pose sur la cellule choisie.
    ///
    /// La règle (`thumb_travel`) dit *de combien* la marque se déplace ; ce
    /// test dit ce que la fenêtre dessine. La première image est prise dans
    /// l'image même où l'animation naît — l'état de l'animation est créé au
    /// premier placement, donc le premier écart mesuré vaut zéro — et la
    /// seconde après le voyage, quand la marque est posée.
    #[gpui::test]
    fn the_fill_travels_from_the_cell_it_left_to_the_cell_it_chose(
        cx: &mut gpui::TestAppContext,
    ) {
        use gpui::prelude::*;
        use gpui::{div, point, px, size, Context, Render, Window};

        struct Fixture {
            theme: Theme,
            marker: Option<SelectionMarker>,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                div().size_full().child(ViewModeSwitcher::render(
                    ViewModeSwitcherProps {
                        current_mode: PackageViewMode::Table,
                        theme: &theme,
                        reduce_motion: false,
                        marker: self.marker,
                        on_select_mode: std::rc::Rc::new(|_, _, _| {}),
                    },
                ))
            }
        }

        let cx = cx.add_empty_window();
        let fixture = cx.new(|_| Fixture {
            theme: Theme::dark(),
            // La marque vient de la cellule des cartes et va à celle du
            // tableau : elle a donc quitté la première.
            marker: Some(SelectionMarker {
                index: 1,
                from: Some(0),
                epoch: 7,
            }),
        });

        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(400.0), px(120.0)),
                |_window, _cx| fixture.clone().into_element(),
            );
        };

        draw(cx);
        let departure = cx
            .debug_bounds("view_mode_fill")
            .expect("la marque est dessinée");

        // Le voyage dure 220 ms ; on attend plus, et on dessine encore.
        std::thread::sleep(std::time::Duration::from_millis(320));
        draw(cx);
        let arrival = cx
            .debug_bounds("view_mode_fill")
            .expect("la marque est dessinée");

        assert_eq!(
            departure.origin.x,
            px(0.0),
            "la marque part de la cellule qu'elle quittait"
        );
        assert_eq!(
            arrival.origin.x,
            px(VIEW_MODE_STRIDE),
            "et se pose sur la cellule choisie"
        );
        assert_eq!(
            arrival.origin.x - departure.origin.x,
            px(VIEW_MODE_STRIDE),
            "la marque a voyagé d'une cellule à l'autre"
        );
    }
}
