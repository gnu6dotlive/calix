use crate::components::sidebar::{Sidebar, SidebarProps};
use crate::state::package_store::PackageStore;
use crate::state::session::{AppSession, NavDestination, SessionEvent};
use crate::state::{AnimatedScalar, MotionDurations};
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use crate::visual_style::VisualStyleId;
use gpui::*;
use std::rc::Rc;
use std::time::Instant;

/// Vue d'état isolée pour la barre latérale de navigation.
/// Possède son propre cycle d'animation continue (AnimatedScalar),
/// garantissant que le rafraîchissement à 60/120 FPS ne déclenche pas le re-render du Workspace racine.
pub struct SidebarView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
    pub width_scalar: AnimatedScalar,
    pub theme: Theme,
    pub visual_style: VisualStyleId,
    pub reduce_motion: bool,
    _session_sub: Subscription,
}

impl SidebarView {
    pub fn new(
        session: Entity<AppSession>,
        store: Entity<PackageStore>,
        theme: Theme,
        reduce_motion: bool,
        visual_style: VisualStyleId,
        cx: &mut Context<Self>,
    ) -> Self {
        let is_collapsed = session.read(cx).sidebar_collapsed;
        let initial_width = if is_collapsed {
            UiMetrics::SIDEBAR_COLLAPSED
        } else {
            UiMetrics::SIDEBAR_EXPANDED
        };
        let width_scalar = AnimatedScalar::new(initial_width);

        let session_sub = cx.subscribe(&session, |this, _session, event, cx| {
            if let SessionEvent::SidebarToggled(collapsed) = event {
                let target = if *collapsed {
                    UiMetrics::SIDEBAR_COLLAPSED
                } else {
                    UiMetrics::SIDEBAR_EXPANDED
                };
                this.width_scalar.retarget(
                    target,
                    MotionDurations::STANDARD,
                    Instant::now(),
                    this.reduce_motion,
                );
                cx.notify();
            }
        });

        Self {
            session,
            store,
            width_scalar,
            theme,
            visual_style,
            reduce_motion,
            _session_sub: session_sub,
        }
    }

    pub fn set_reduce_motion(&mut self, reduce_motion: bool, cx: &mut Context<Self>) {
        self.reduce_motion = reduce_motion;
        if reduce_motion {
            self.width_scalar.snap(self.width_scalar.target_value);
        }
        cx.notify();
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        cx.notify();
    }

    pub fn set_visual_style(&mut self, visual_style: VisualStyleId, cx: &mut Context<Self>) {
        self.visual_style = visual_style;
        cx.notify();
    }
}

impl Render for SidebarView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let animating = self.width_scalar.update(Instant::now());
        if animating || self.width_scalar.is_active() {
            window.request_animation_frame();
        }

        let current_width = self.width_scalar.current;
        let (destination, is_collapsed) = {
            let s = self.session.read(cx);
            (s.destination, s.sidebar_collapsed)
        };
        let updates_count = self.store.read(cx).updates_count;

        let session_dest = self.session.clone();
        let session_toggle = self.session.clone();

        Sidebar::render(SidebarProps {
            active_destination: destination,
            updates_count,
            is_collapsed,
            current_width,
            theme: &self.theme,
            visual_style: self.visual_style,
            on_select_destination: Rc::new(move |dest: NavDestination, _w, cx| {
                session_dest.update(cx, |s, cx| s.set_destination(dest, cx));
            }),
            on_toggle_collapse: Rc::new(move |_w, cx| {
                session_toggle.update(cx, |s, cx| s.toggle_sidebar(cx));
            }),
        })
    }
}
