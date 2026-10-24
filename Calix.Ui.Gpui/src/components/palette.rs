//! The command palette: every action of the registry, found by its words.
//!
//! Skwir's lesson: every user action is one command, which the keymap, the
//! palette and the menus share. Here the palette is a window on the registry
//! (`keys::matching`); choosing a line dispatches the action GPUI's keymap
//! would, so the palette cannot do anything a key does not, nor differently.
//!
//! A popup owns the keyboard: the palette takes the focus when it opens, its
//! arrows walk its lines (bound in its own key context, deeper than the
//! window's), and the focus goes back where it was when it closes.

use crate::components::cheatsheet::Cheatsheet;
use crate::components::search_input::{self, SearchEvent, SearchInputView};
use crate::state::keys::{self, KeyAction, PALETTE_CONTEXT};
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use gpui::prelude::FluentBuilder;
use gpui::*;

actions!(command_palette, [PaletteNext, PalettePrevious]);

/// The palette's own keys: the arrows walk its lines, from inside its field.
pub fn bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("down", PaletteNext, Some(PALETTE_CONTEXT)),
        KeyBinding::new("up", PalettePrevious, Some(PALETTE_CONTEXT)),
    ]
}

/// What the palette tells the window.
pub enum PaletteEvent {
    /// The user chose a line: the window closes the palette and runs it.
    Run(&'static KeyAction),
    /// The user left without choosing.
    Dismissed,
}

pub struct CommandPalette {
    field: Entity<SearchInputView>,
    query: String,
    highlighted: usize,
    scroll: ScrollHandle,
    _field_events: Subscription,
}

impl EventEmitter<PaletteEvent> for CommandPalette {}

impl CommandPalette {
    pub fn new(reduce_motion: bool, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let field = cx.new(|cx| SearchInputView::new("Run a command…", reduce_motion, cx));
        let field_events = cx.subscribe(&field, |this, _field, event: &SearchEvent, cx| {
            match event {
                SearchEvent::Changed(text) => {
                    this.query = text.clone();
                    this.highlighted = 0;
                    this.scroll.scroll_to_item(0);
                    cx.notify();
                }
                SearchEvent::Submitted(_) => this.run_highlighted(cx),
                SearchEvent::Cleared => {}
            }
        });
        field.update(cx, |field, cx| field.focus(window, cx));
        Self {
            field,
            query: String::new(),
            highlighted: 0,
            scroll: ScrollHandle::new(),
            _field_events: field_events,
        }
    }

    fn offered(&self) -> Vec<&'static KeyAction> {
        keys::matching(&self.query)
    }

    fn step(&mut self, forward: bool, cx: &mut Context<Self>) {
        let count = self.offered().len();
        if count == 0 {
            return;
        }
        self.highlighted = if forward {
            (self.highlighted + 1) % count
        } else {
            (self.highlighted + count - 1) % count
        };
        self.scroll.scroll_to_item(self.highlighted);
        cx.notify();
    }

    fn run_highlighted(&mut self, cx: &mut Context<Self>) {
        if let Some(action) = self.offered().get(self.highlighted).copied() {
            cx.emit(PaletteEvent::Run(action));
        }
    }

    fn render_line(
        &self,
        index: usize,
        action: &'static KeyAction,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let highlighted = index == self.highlighted;
        let (fill, text) = if highlighted {
            (theme.selection_bg, theme.selection_text)
        } else {
            (gpui::transparent_black().into(), theme.text_primary)
        };
        let hover = theme.bg_surface_hover;
        div()
            // A line is known by the action it shows, never by where it stands:
            // filtering moves lines, and an id by index would hand one line's
            // press to the next.
            .id(SharedString::from(format!("palette:{}", action.id)))
            .role(Role::ListBoxOption)
            .aria_label(action.label)
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .py_1p5()
            .rounded_md()
            .cursor_pointer()
            .bg(fill)
            .when(!highlighted, |line| line.hover(move |s| s.bg(hover)))
            .on_mouse_move(cx.listener(move |this, _: &MouseMoveEvent, _, cx| {
                if this.highlighted != index {
                    this.highlighted = index;
                    cx.notify();
                }
            }))
            .on_click(cx.listener(move |_this, _: &ClickEvent, _, cx| {
                cx.emit(PaletteEvent::Run(action));
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_sm()
                    .text_color(text)
                    .child(action.label),
            )
            .child(
                div()
                    .flex_none()
                    .text_xs()
                    .text_color(if highlighted { text } else { theme.text_muted })
                    .child(action.group),
            )
            .child(Cheatsheet::render_keycap(&action.binding, theme))
    }
}

impl Render for CommandPalette {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx);
        let offered = self.offered();
        let lines: Vec<AnyElement> = offered
            .iter()
            .enumerate()
            .map(|(index, action)| self.render_line(index, action, &theme, cx).into_any_element())
            .collect();
        // Empty is a state: say why, and what is still possible.
        let empty = offered.is_empty().then(|| {
            div()
                .px_3()
                .py_4()
                .text_sm()
                .text_color(theme.text_muted)
                .child(format!(
                    "No command matches “{}”. Escape clears the text.",
                    self.query.trim()
                ))
        });

        div()
            .id("command_palette_overlay")
            .absolute()
            .inset_0()
            .flex()
            .flex_col()
            .items_center()
            .pt(relative(0.12))
            .child(
                div()
                    .id("command_palette")
                    .key_context(PALETTE_CONTEXT)
                    .role(Role::Dialog)
                    .aria_label("Run a command")
                    .occlude()
                    .w(px(560.0))
                    .max_w(relative(0.9))
                    .max_h(relative(0.7))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .p_2()
                    .rounded_lg()
                    .shadow_lg()
                    .border_1()
                    .border_color(theme.border)
                    .bg(theme.bg_app)
                    .on_action(cx.listener(|this, _: &PaletteNext, _, cx| this.step(true, cx)))
                    .on_action(cx.listener(|this, _: &PalettePrevious, _, cx| this.step(false, cx)))
                    // Escape leaves the palette before the field would clear
                    // its text: the field's text is a filter, not the point.
                    .capture_action(cx.listener(|_this, _: &search_input::Escape, _, cx| {
                        cx.stop_propagation();
                        cx.emit(PaletteEvent::Dismissed);
                    }))
                    .on_mouse_down_out(cx.listener(|_this, _: &MouseDownEvent, _, cx| {
                        cx.emit(PaletteEvent::Dismissed);
                    }))
                    .child(
                        div()
                            .flex_none()
                            .h(px(UiMetrics::SEARCH_INPUT_HEIGHT))
                            .child(self.field.clone()),
                    )
                    .child(
                        div()
                            .id("command_palette_lines")
                            .role(Role::ListBox)
                            .aria_label("Commands")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .flex()
                            .flex_col()
                            .children(lines)
                            .children(empty),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    struct Host {
        palette: Entity<CommandPalette>,
        events: Vec<&'static str>,
        _sub: Subscription,
    }

    impl Render for Host {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().relative().child(self.palette.clone())
        }
    }

    fn open(cx: &mut gpui::TestAppContext) -> (Entity<Host>, &mut gpui::VisualTestContext) {
        cx.update(|cx| {
            crate::theme::Theme::install(crate::theme::Theme::dark(), cx);
            cx.bind_keys(bindings());
        });
        let (host, cx) = cx.add_window_view(|window, cx| {
            let palette = cx.new(|cx| CommandPalette::new(true, window, cx));
            let sub = cx.subscribe(&palette, |host: &mut Host, _, event: &PaletteEvent, _| {
                host.events.push(match event {
                    PaletteEvent::Run(action) => action.id,
                    PaletteEvent::Dismissed => "dismissed",
                });
            });
            Host {
                palette,
                events: Vec::new(),
                _sub: sub,
            }
        });
        cx.run_until_parked();
        (host, cx)
    }

    #[gpui::test]
    fn the_arrows_walk_the_lines_and_wrap(cx: &mut gpui::TestAppContext) {
        let (host, cx) = open(cx);
        let palette = host.read_with(cx, |host, _| host.palette.clone());
        cx.simulate_keystrokes("down down");
        palette.read_with(cx, |p, _| assert_eq!(p.highlighted, 2));
        cx.simulate_keystrokes("up up up");
        let last = keys::matching("").len() - 1;
        palette.read_with(cx, |p, _| assert_eq!(p.highlighted, last));
    }

    #[gpui::test]
    fn the_text_filters_and_choosing_runs_the_first_line(cx: &mut gpui::TestAppContext) {
        let (host, cx) = open(cx);
        let palette = host.read_with(cx, |host, _| host.palette.clone());
        let field = palette.read_with(cx, |p, _| p.field.clone());
        field.update(cx, |field, cx| {
            field.set_text("upd", cx);
            cx.emit(SearchEvent::Changed("upd".into()));
        });
        cx.run_until_parked();
        palette.update(cx, |p, cx| p.run_highlighted(cx));
        cx.run_until_parked();
        host.read_with(cx, |host, _| assert_eq!(host.events, vec!["rail.updates"]));
    }

    #[gpui::test]
    fn the_palette_draws_its_empty_state(cx: &mut gpui::TestAppContext) {
        let (host, cx) = open(cx);
        let palette = host.read_with(cx, |host, _| host.palette.clone());
        palette.update(cx, |p, cx| {
            p.query = "zebra".into();
            cx.notify();
        });
        cx.run_until_parked();
        palette.read_with(cx, |p, _| assert!(p.offered().is_empty()));
        // Choosing nothing runs nothing.
        palette.update(cx, |p, cx| p.run_highlighted(cx));
        cx.run_until_parked();
        host.read_with(cx, |host, _| assert!(host.events.is_empty()));
    }
}
