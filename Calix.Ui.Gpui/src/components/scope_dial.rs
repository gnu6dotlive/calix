use crate::icons::AppIcon;
use crate::state::motion::{ease_spring, MotionDurations};
use crate::state::NavScope;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

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

/// The geometry of the dial, in page units.
///
/// The dial is mounted beside the scope's name, in the header: the name is its
/// label, so it carries none of its own, and it fits in a band without
/// hollowing it out above and below.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialGeometry {
    pub width: f32,
    pub height: f32,
    pub face: f32,
    pub face_top: f32,
    pub radius: f32,
    pub detent: f32,
    pub icon: f32,
    pub hub: f32,
}

impl DialGeometry {
    pub const fn new() -> Self {
        Self {
            width: 72.0,
            height: 72.0,
            face: 56.0,
            face_top: 12.0,
            radius: 31.0,
            detent: 16.0,
            icon: 10.0,
            hub: 6.0,
        }
    }

    pub fn centre_x(&self) -> f32 {
        self.width / 2.0
    }

    pub fn centre_y(&self) -> f32 {
        self.face_top + self.face / 2.0
    }

    pub fn face_left(&self) -> f32 {
        (self.width - self.face) / 2.0
    }
}

impl Default for DialGeometry {
    fn default() -> Self {
        Self::new()
    }
}

/// The dial: the entity that holds the needle's angle.
pub struct ScopeDialView {
    scope: NavScope,
    /// The detent the needle left: its throw starts there.
    needle_from: f32,
    /// One per throw. The needle is animated by its element
    /// (`with_animation`), keyed by this count: the view never re-renders
    /// itself to move it.
    needle_throw: usize,
    reduce_motion: bool,
    on_select: ScopeSelectHandler,
}

impl ScopeDialView {
    pub fn new(
        scope: NavScope,
        reduce_motion: bool,
        on_select: ScopeSelectHandler,
    ) -> Self {
        Self {
            scope,
            needle_from: scope_angle(scope),
            needle_throw: 0,
            reduce_motion,
            on_select,
        }
    }

    /// The scope changed elsewhere: the needle is thrown at the new detent.
    pub fn set_scope(&mut self, scope: NavScope, cx: &mut Context<Self>) {
        if scope == self.scope {
            return;
        }
        self.throw_needle(scope);
        cx.notify();
    }

    fn throw_needle(&mut self, scope: NavScope) {
        self.needle_from = scope_angle(self.scope);
        self.scope = scope;
        self.needle_throw += 1;
    }

    /// The needle is thrown at a detent.
    pub fn retarget(&mut self, scope: NavScope, cx: &mut Context<Self>) {
        if scope == self.scope {
            return;
        }
        self.throw_needle(scope);
        cx.notify();
    }
}

impl Render for ScopeDialView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::of(cx);
        let angle = scope_angle(self.scope);
        let from = self.needle_from;
        let throw = self.needle_throw;
        let still = self.reduce_motion || throw == 0;
        let selected = self.scope;
        let g = DialGeometry::new();

        // The face: a machined disc, standing proud of the plate. It sits
        // low in its box, so that the arc above it has room.
        let face = div()
            .id("scope_dial_face")
            .absolute()
            .left(px(g.face_left()))
            .top(px(g.face_top))
            .size(px(g.face))
            .rounded_full()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            // The knob stands proud of the aluminium: the one object of the
            // chassis that casts a shadow of its own.
            .shadow_md()
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
                    .child({
                        let needle = svg()
                            .path(AppIcon::DialNeedle.path())
                            .size(px(g.face))
                            .text_color(theme.accent_text);
                        if still {
                            needle
                                .with_transformation(Transformation::rotate(gpui::radians(
                                    angle.to_radians(),
                                )))
                                .into_any_element()
                        } else {
                            // The needle overshoots the detent and comes back:
                            // an instrument overshoots, a switch does not.
                            needle
                                .with_animation(
                                    ("scope_dial_needle", throw),
                                    Animation::new(MotionDurations::EMPHASIS)
                                        .with_easing(ease_spring),
                                    move |needle, delta| {
                                        let at = from + (angle - from) * delta;
                                        needle.with_transformation(Transformation::rotate(
                                            gpui::radians(at.to_radians()),
                                        ))
                                    },
                                )
                                .into_any_element()
                        }
                    }),
            )
            .child(
                // The hub: the needle's pivot.
                div()
                    .absolute()
                    .size(px(g.hub))
                    .rounded_full()
                    .bg(theme.accent)
                    .top(px(g.centre_y() - g.hub / 2.0))
                    .left(px(g.centre_x() - g.hub / 2.0)),
            );

        // The detents: three marks on the upper arc, each carrying the sign
        // of its scope. The names are long — they are written under the dial,
        // once, for the scope the needle is on.
        let mut detents = Vec::new();
        for scope in NavScope::ALL {
            let scope_angle = scope_angle(*scope);
            let radians = scope_angle.to_radians();
            let radius = g.radius;
            let x = g.centre_x() + radius * radians.sin();
            let y = g.centre_y() - radius * radians.cos();
            let is_selected = *scope == selected;
            let on_select = self.on_select.clone();
            let entity = cx.entity();
            detents.push(
                div()
                    .id(detent_id(*scope))
                    .absolute()
                    .left(px(x - g.detent / 2.0))
                    .top(px(y - g.detent / 2.0))
                    .w(px(g.detent))
                    .h(px(g.detent))
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
                    .on_click({
                        let on_select = on_select.clone();
                        let entity = entity.clone();
                        move |event, window, cx| {
                            if event.is_keyboard() {
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
                            .size(px(g.icon))
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
            .w(px(g.width))
            .h(px(g.height))
            .flex_shrink_0()
            .child(face)
            .children(detents)
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
