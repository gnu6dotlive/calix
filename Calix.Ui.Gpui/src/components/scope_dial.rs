use crate::icons::AppIcon;
use crate::state::motion::{ease_spring, AnimatedScalar, MotionDurations};
use crate::state::NavScope;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;
use std::time::Instant;

pub type ScopeSelectHandler = Rc<dyn Fn(NavScope, &mut Window, &mut App) + 'static>;

/// The dial of the scope: one object with three positions.
///
/// The sources are a bank of seven equal positions; the scope is one object
/// with three. They must not look alike, and they must never share a
/// control. The dial stands on its own in the control strip, above the
/// readout it governs, and turning it never changes a source.
///
/// The needle is thrown at the detent and settles with a slight overshoot: an
/// instrument overshoots, a switch does not.
/// The element ids of the dial, one per detent. They must outlive the frame.
fn detent_id(scope: NavScope) -> &'static str {
    match scope {
        NavScope::Results => "scope_dial_detent_results",
        NavScope::Installed => "scope_dial_detent_installed",
        NavScope::Updates => "scope_dial_detent_updates",
    }
}

/// The angle of a scope, in degrees, from the vertical.
///
/// The three positions are on the upper arc: the needle points left, up, or
/// right.
pub fn scope_angle(scope: NavScope) -> f32 {
    match scope {
        NavScope::Results => -38.0,
        NavScope::Installed => 0.0,
        NavScope::Updates => 38.0,
    }
}

/// The dial: the entity that holds the needle's angle.
pub struct ScopeDialView {
    scope: NavScope,
    needle: AnimatedScalar,
    theme: Theme,
    reduce_motion: bool,
    on_select: ScopeSelectHandler,
}

impl ScopeDialView {
    pub fn new(
        scope: NavScope,
        theme: Theme,
        reduce_motion: bool,
        on_select: ScopeSelectHandler,
    ) -> Self {
        let mut needle = AnimatedScalar::new(scope_angle(scope));
        // The needle overshoots the detent and comes back: an instrument
        // overshoots, a switch does not.
        needle.easing = ease_spring;
        Self {
            scope,
            needle,
            theme,
            reduce_motion,
            on_select,
        }
    }

    /// The scope changed elsewhere: the needle is thrown at the new detent.
    pub fn set_scope(&mut self, scope: NavScope, cx: &mut Context<Self>) {
        if scope == self.scope {
            return;
        }
        self.scope = scope;
        self.needle.retarget(
            scope_angle(scope),
            MotionDurations::EMPHASIS,
            Instant::now(),
            self.reduce_motion,
        );
        cx.notify();
    }

    /// The needle is thrown at a detent.
    pub fn retarget(&mut self, scope: NavScope, cx: &mut Context<Self>) {
        if scope == self.scope {
            return;
        }
        self.scope = scope;
        self.needle.retarget(
            scope_angle(scope),
            MotionDurations::EMPHASIS,
            Instant::now(),
            self.reduce_motion,
        );
        cx.notify();
    }
}

impl Render for ScopeDialView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The needle's throw: the frame gives back the next position.
        let animating = self.needle.update(Instant::now());
        if animating {
            cx.notify();
        }

        let theme = self.theme;
        let angle = self.needle.current;
        let selected = self.scope;

        // The face: a machined disc, standing proud of the plate. It sits
        // low in its box, so that the arc above it has room.
        let face = div()
            .id("scope_dial_face")
            .absolute()
            .left(px(10.0))
            .top(px(30.0))
            .size(px(76.0))
            .rounded_full()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                // The needle, thrown at the detent. The SVG is painted upside
                // down — the engine flips it — so the file's needle points
                // down and the rotation brings it up.
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path(AppIcon::DialNeedle.path())
                            .size(px(44.0))
                            .text_color(theme.accent)
                            .with_transformation(Transformation::rotate(
                                gpui::radians(angle.to_radians()),
                            )),
                    ),
            )
            .child(
                // The hub: the needle's pivot.
                div()
                    .absolute()
                    .size(px(8.0))
                    .rounded_full()
                    .bg(theme.accent)
                    .top(px(34.0))
                    .left(px(34.0)),
            );

        // The detents: three marks on the upper arc, each carrying the sign
        // of its scope. The names are long — they are written under the dial,
        // once, for the scope the needle is on.
        let mut detents = Vec::new();
        for scope in NavScope::ALL {
            let scope_angle = scope_angle(*scope);
            let radians = scope_angle.to_radians();
            let radius = 42.0_f32;
            let x = 48.0 + radius * radians.sin();
            let y = 68.0 - radius * radians.cos();
            let is_selected = *scope == selected;
            let on_select = self.on_select.clone();
            let entity = cx.entity();
            detents.push(
                div()
                    .id(detent_id(*scope))
                    .absolute()
                    .left(px(x - 10.0))
                    .top(px(y - 10.0))
                    .w(px(20.0))
                    .h(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(if is_selected {
                        theme.bg_surface_hover
                    } else {
                        gpui::rgba(0x00000000)
                    })
                    .cursor_pointer()
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.bg(theme.bg_surface_hover))
                    .hover(move |s| s.bg(theme.bg_surface_hover))
                    .on_mouse_down(MouseButton::Left, {
                        let on_select = on_select.clone();
                        let entity = entity.clone();
                        move |_e, window, cx| {
                            // The needle is thrown at the detent, and the
                            // scope changes with it.
                            entity.update(cx, |view, cx| {
                                view.retarget(*scope, cx);
                            });
                            on_select(*scope, window, cx);
                        }
                    })
                    .on_key_down({
                        let on_select = on_select.clone();
                        let entity = entity.clone();
                        move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                entity.update(cx, |view, cx| {
                                    view.retarget(*scope, cx);
                                });
                                on_select(*scope, window, cx);
                            }
                        }
                    })
                    .child(
                        svg()
                            .path(scope.icon().path())
                            .size(px(13.0))
                            .text_color(if is_selected {
                                theme.accent
                            } else {
                                theme.text_muted
                            }),
                    ),
            );
        }

        div()
            .id("scope_dial")
            .relative()
            .w(px(96.0))
            .h(px(124.0))
            .flex_shrink_0()
            .child(face)
            .children(detents)
            .child(
                // The name of the scope the needle is on.
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(108.0))
                    .w(px(96.0))
                    .text_xs()
                    .text_align(TextAlign::Center)
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .child(selected.label()),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_three_positions_are_on_the_upper_arc() {
        assert!(scope_angle(NavScope::Results) < 0.0);
        assert_eq!(scope_angle(NavScope::Installed), 0.0);
        assert!(scope_angle(NavScope::Updates) > 0.0);
    }
}
