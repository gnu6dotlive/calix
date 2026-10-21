use crate::components::sidebar::{Sidebar, SidebarProps};
use crate::state::motion::{advance_marker, SelectionMarker};
use crate::state::package_store::PackageStore;
use crate::state::session::{AppSession, RailEntry, SessionEvent};
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
    /// La place manque : la colonne se replie, quelle que soit la forme que
    /// l'utilisateur lui a donnée. C'est la règle de l'échelle, et elle se
    /// défait quand la fenêtre reprend de la place.
    pub forced_collapsed: bool,
    /// Le trait de la sélection : la rangée qu'il marque, celle qu'il quitte.
    pub rail_marker: Option<SelectionMarker>,
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
            forced_collapsed: false,
            rail_marker: None,
            theme,
            visual_style,
            reduce_motion,
            _session_sub: session_sub,
        }
    }

    /// La place manque : la colonne se replie. Le propriétaire de la mise en
    /// page le décide, à partir de la largeur qu'il a.
    pub fn set_forced_collapsed(&mut self, forced: bool, cx: &mut Context<Self>) {
        if self.forced_collapsed != forced {
            self.forced_collapsed = forced;
            cx.notify();
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let animating = self.width_scalar.update(Instant::now());
        if animating || self.width_scalar.is_active() {
            // The view asks for the next frame itself: a window animation frame
            // would notify the enclosing view instead of this one, and panics
            // when the view is laid out on its own.
            cx.notify();
        }

        let current_width = self.width_scalar.current;
        let (place, scope, user_collapsed) = {
            let s = self.session.read(cx);
            (s.place, s.scope, s.sidebar_collapsed)
        };
        // La forme de la colonne : celle de l'utilisateur, ou celle que la
        // place impose. Elle s'animationne comme un voyage : la colonne se
        // replie, elle ne saute pas.
        let is_collapsed = user_collapsed || self.forced_collapsed;
        let target_width = if is_collapsed {
            UiMetrics::SIDEBAR_COLLAPSED
        } else {
            UiMetrics::SIDEBAR_EXPANDED
        };
        if (self.width_scalar.target_value - target_width).abs() > 0.5 {
            self.width_scalar.retarget(
                target_width,
                MotionDurations::STANDARD,
                Instant::now(),
                self.reduce_motion,
            );
        }
        let updates_count = self.store.read(cx).updates_count;

        // La sélection du rail est un objet qui voyage : quand le périmètre
        // change, le trait part de l'entrée quittée et se pose sur l'entrée
        // choisie. La même loi que le porteur de la liste, et le même porteur.
        let selected = RailEntry::ALL
            .iter()
            .position(|entry| entry.is_active(place, scope));
        let (rail_marker, _) = advance_marker(self.rail_marker, selected);
        self.rail_marker = rail_marker;

        let session_dest = self.session.clone();
        let session_toggle = self.session.clone();

        Sidebar::render(SidebarProps {
            active_place: place,
            active_scope: scope,
            updates_count,
            is_collapsed,
            current_width,
            theme: &self.theme,
            visual_style: self.visual_style,
            on_select: Rc::new(move |entry: RailEntry, _w, cx| {
                session_dest.update(cx, |s, cx| {
                    if let Some(scope) = entry.scope {
                        s.set_scope(scope, cx);
                    }
                    s.set_place(entry.place, cx);
                });
            }),
            on_toggle_collapse: Rc::new(move |_w, cx| {
                session_toggle.update(cx, |s, cx| s.toggle_sidebar(cx));
            }),
            marker: rail_marker,
            reduce_motion: self.reduce_motion,
        })
    }
}
