use crate::components::sidebar::{Sidebar, SidebarProps};
use crate::state::motion::{advance_marker, SelectionMarker};
use crate::state::package_store::PackageStore;
use crate::state::session::{AppSession, RailEntry, SessionEvent};
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use crate::visual_style::VisualStyleId;
use gpui::*;
use std::rc::Rc;

/// Vue d'état isolée pour la barre latérale de navigation.
///
/// Sa largeur change d'un coup : une colonne qui s'anime redessine la fenêtre
/// à chaque image et fait onduler le catalogue à côté d'elle. Rien ici ne se
/// re-notifie depuis `render`.
pub struct SidebarView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
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
        let session_sub = cx.subscribe(&session, |_this, _session, event, cx| {
            if let SessionEvent::SidebarToggled(_) = event {
                cx.notify();
            }
        });

        Self {
            session,
            store,
            rail_marker: None,
            theme,
            visual_style,
            reduce_motion,
            _session_sub: session_sub,
        }
    }

    pub fn set_reduce_motion(&mut self, reduce_motion: bool, cx: &mut Context<Self>) {
        self.reduce_motion = reduce_motion;
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
        let (place, scope, user_collapsed) = {
            let s = self.session.read(cx);
            (s.place, s.scope, s.sidebar_collapsed)
        };
        // La forme de la colonne : celle de l'utilisateur, ou celle que la
        // place impose. La colonne lit elle-même la fenêtre : c'est une
        // décision prise en dessinant, sans rien écrire, et personne n'a à la
        // lui pousser depuis un autre `render`.
        let forced_collapsed = crate::views::package_workstation::spine_is_forced_collapsed(
            window.viewport_size().width / px(1.0),
        );
        let is_collapsed = user_collapsed || forced_collapsed;
        let current_width = if is_collapsed {
            UiMetrics::SIDEBAR_COLLAPSED
        } else {
            UiMetrics::SIDEBAR_EXPANDED
        };
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
