use crate::backend::models::UnifiedPackage;
use crate::components::menu::MenuLifecycle;
use crate::components::package_card::{PackageCard, PackageCardProps};
use crate::components::package_table::PackageTable;
use crate::components::query_workbench::{
    ActiveMenuState, QueryWorkbench, QueryWorkbenchProps, ScopeAction, WorkbenchMenu,
};
use crate::components::scope_dial::ScopeDialView;
use crate::components::search_input::SearchInputView;
use crate::components::source_bank::{SourceBank, SourceBankProps, SourceRetryHandler};
use crate::components::unified_search::UnifiedSearch;
use crate::state::console::{ConsoleEvent, ConsoleModel};
use crate::state::motion::{advance_marker, SelectionMarker};
use crate::state::query::{PackageStateFilter, SortMode, SourceScope};
use crate::state::{
    also_in, canonical_install_command, source_axis, AlsoIn, AppSession, InspectorTab, NavScope,
    PackageKey, PackageStore, PackageViewMode, SessionEvent, ToastCenter, ToastKind,
};
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use crate::views::inspector::{PackageInspectorProps, PackageInspectorView, QuietPlate};
use crate::visual_style::VisualStyleId;
use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui::{uniform_list, UniformListScrollHandle};
use std::rc::Rc;
use std::time::Duration;

pub type PackageMutationHandler =
    Rc<dyn Fn(&UnifiedPackage, bool /* is_install */, &mut Window, &mut App) + 'static>;
pub type SystemUpgradeHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type ActionReloadHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type DetailsRetryHandler = Rc<dyn Fn(&PackageKey, &mut Window, &mut App) + 'static>;

/// What the nameplate's edge carries while it is dragged. Nothing follows the
/// pointer: the edge itself moves.
pub struct NameplateEdge;

impl Render for NameplateEdge {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// The drag of the nameplate's edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NameplateDragState {
    pub start_height: f32,
    pub start_pointer_y: f32,
}

/// The nameplate's bounds: it never swallows the readout, never disappears.
///
/// The plate is read from the top down in full-width bands. The nameplate is
/// the caption of the readout: below it, always, between a floor and a
/// share.
///
/// The band is the space the station really has — not the window. The plate
/// takes what the user asks for, up to a ceiling computed *after* the bank
/// and the catalogue's own floor: the catalogue keeps its rows, the plate
/// takes the rest.
pub fn nameplate_bounds(band_height: f32) -> (f32, f32) {
    let min = UiMetrics::NAMEPLATE_MIN_HEIGHT;
    // The band is the one the catalogue and the plate share, under the bank:
    // what the drag reads from its own bounds.
    let after_the_catalogue =
        band_height - UiMetrics::SPLITTER_THICKNESS - UiMetrics::CATALOGUE_MIN_HEIGHT;
    // Two ceilings: the space that is left once the catalogue has kept its
    // rows, and the share of the band a context surface may take.
    let max = after_the_catalogue
        .min(band_height * UiMetrics::NAMEPLATE_MAX_RATIO)
        .max(min);
    (min, max)
}

/// The height of the nameplate, bounded.
pub fn nameplate_height(band_height: f32, requested: f32) -> f32 {
    let (min, max) = nameplate_bounds(band_height);
    requested.clamp(min, max)
}

/// The height of the nameplate after a drag: pulling up makes it grow.
pub fn nameplate_height_after_drag(
    drag_start_height: f32,
    drag_start_pointer_y: f32,
    current_pointer_y: f32,
    content_height: f32,
) -> f32 {
    let delta_y = current_pointer_y - drag_start_pointer_y;
    nameplate_height(content_height, drag_start_height - delta_y)
}


/// La largeur de la zone de contenu : la largeur *mappée* de la fenêtre, moins le rail.
///
/// La largeur que l'application a *demandée* (`window.window_bounds()`) n'est pas
/// celle que le compositeur lui accorde : un compositeur à tuiles redimensionne la
/// fenêtre quand le focus change, et la demande reste celle du lancement. Mesurer
/// sur la demande faisait tenir l'inspecteur à $137px$ — sous son plancher de
/// $320px$ — dans une fenêtre de $1056px$, puis à $1622px$ — au-dessus de son
/// plafond de $520px$ — dans une fenêtre de $2542px$. C'est la taille mappée,
/// `viewport_size()`, qui fait foi.
pub fn content_width(viewport_width: f32, sidebar_width: f32) -> f32 {
    (viewport_width - sidebar_width).max(0.0)
}

/// La colonne de navigation se replie-t-elle ?
///
/// Elle se replie quand le catalogue n'a plus la place qu'il doit garder :
/// la colonne est la première surface qui peut rétrécir, et elle ne
/// disparaît jamais — elle garde ses cinq destinations.
///
/// C'est une règle de l'échelle, pas un état : quand la fenêtre reprend de
/// la place, la colonne reprend la forme que l'utilisateur lui avait
/// donnée.
pub fn spine_is_forced_collapsed(viewport_width: f32) -> bool {
    content_width(viewport_width, UiMetrics::SIDEBAR_EXPANDED) < UiMetrics::CATALOGUE_MIN_WIDTH
}

/// La largeur de la colonne, une fois la règle appliquée.
pub fn effective_sidebar_width(user_collapsed: bool, viewport_width: f32) -> f32 {
    if user_collapsed || spine_is_forced_collapsed(viewport_width) {
        UiMetrics::SIDEBAR_COLLAPSED
    } else {
        UiMetrics::SIDEBAR_EXPANDED
    }
}

/// La position du porteur dans le renfoncement : le haut de la liste, puis la
/// ligne, moins ce que la liste a défilé.
///
/// La ligne peut être fractionnaire : la barre voyage entre deux lignes.
pub fn carrier_top(header_height: f32, row_height: f32, row: f32, scroll_y: f32) -> f32 {
    header_height + row * row_height - scroll_y
}

/// L'ordre des périmètres sur le cadran : la gauche, le haut, la droite. Le
/// sens du changement décide du côté par lequel l'entrée arrive.
pub fn scope_order(scope: NavScope) -> i32 {
    match scope {
        NavScope::Results => 0,
        NavScope::Installed => 1,
        NavScope::Updates => 2,
    }
}

/// Le voyage de l'entrée du périmètre, en pixels. La courbe de sortie dépense
/// l'essentiel de sa course au début : la surface doit donc parcourir une part
/// de sa propre largeur pour que le mouvement se voie.
pub const SCOPE_ENTRY_TRAVEL: f32 = 44.0;

/// Le voyage de l'entrée de la plaque, en pixels.
pub const PLATE_ENTRY_TRAVEL: f32 = 22.0;

pub struct PackageWorkstationView {
    pub session: Entity<AppSession>,
    pub store: Entity<PackageStore>,
    pub console: Entity<ConsoleModel>,
    pub toast_center: Entity<ToastCenter>,
    /// The height of the nameplate: the caption of the readout, under it.
    pub nameplate_height: f32,
    pub nameplate_drag: Option<NameplateDragState>,
    pub scroll_handle: UniformListScrollHandle,
    /// La sélection est un objet qui voyage : la barre du porteur glisse d'une
    /// ligne à l'autre au lieu de sauter.
    pub marker: Option<SelectionMarker>,
    /// Le commutateur de vue est une troisième surface du même porteur : sa
    /// marque voyage d'une cellule à l'autre.
    pub view_marker: Option<SelectionMarker>,
    /// Le numéro du dernier changement de sélection : la plaque de nom entre
    /// avec la sélection, pas à chaque rendu.
    pub selection_epoch: usize,
    /// Le périmètre précédent : il donne la direction de l'entrée.
    pub last_scope: NavScope,
    pub scope_epoch: usize,
    pub scope_direction: f32,
    /// L'onglet précédent : il donne le sens de la coupe de la plaque.
    pub last_inspector_tab: InspectorTab,
    pub tab_direction: f32,
    pub on_mutation: Option<PackageMutationHandler>,
    pub on_upgrade_all: Option<SystemUpgradeHandler>,
    pub on_retry_source: Option<SourceRetryHandler>,
    pub on_reload_installed: Option<ActionReloadHandler>,
    pub on_reload_updates: Option<ActionReloadHandler>,
    pub on_retry_details: Option<DetailsRetryHandler>,
    pub copy_cmd_feedback: bool,
    pub is_loading_pkgbuild: bool,
    pub reduce_motion: bool,
    pub theme: Theme,
    pub visual_style: VisualStyleId,
    pub compact: bool,
    pub search_input: Entity<SearchInputView>,
    /// The dial of the scope: one object with three positions.
    pub scope_dial: Entity<ScopeDialView>,
    /// The dial's ear to the session: the needle follows the scope.
    _scope_dial_sub: Subscription,
    pub active_menu: Option<ActiveMenuState>,
    pub menu_epoch: usize,
    pub state_btn_focus: FocusHandle,
    pub sort_btn_focus: FocusHandle,
    pub menu_surface_focus: FocusHandle,
}

#[derive(Clone, Copy, Debug)]
pub struct PackageWorkstationConfig {
    pub theme: Theme,
    pub reduce_motion: bool,
    pub compact: bool,
    pub visual_style: VisualStyleId,
}

impl PackageWorkstationView {
    pub fn new(
        session: Entity<AppSession>,
        store: Entity<PackageStore>,
        console: Entity<ConsoleModel>,
        toast_center: Entity<ToastCenter>,
        config: PackageWorkstationConfig,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.subscribe(&session, |_this, _emitter, _event, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(&store, |_this, _emitter, _event, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(&console, |_this, _emitter, event, cx| match event {
            ConsoleEvent::OperationStarted(_) | ConsoleEvent::OperationFinished(_) => {
                cx.notify();
            }
            _ => {}
        })
        .detach();

        let search_input = cx.new(|cx| {
            SearchInputView::new(
                "Search packages and apps...",
                config.theme,
                config.reduce_motion,
                cx,
            )
        });

        // Le cadran du périmètre. Il est une entité à part entière : l'aiguille
        // a son propre état, et elle se jette sur son cran quand le
        // périmètre change, d'où qu'il vienne.
        let dial_entity = cx.entity();
        let initial_scope = session.read(cx).scope;
        let scope_dial = cx.new(|_cx| {
            ScopeDialView::new(
                initial_scope,
                config.theme,
                config.reduce_motion,
                Rc::new(move |scope, _window, cx| {
                    dial_entity.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.set_scope(scope, cx);
                        });
                    });
                }),
            )
        });

        // Le cadran suit le périmètre, d'où qu'il vienne.
        let dial = scope_dial.clone();
        let dial_subscription = cx.subscribe(&session, move |_this, _emitter, event, cx| {
            if let SessionEvent::ScopeChanged(scope) = event {
                dial.update(cx, |view, cx| {
                    view.set_scope(*scope, cx);
                });
            }
        });

        Self {
            session,
            store,
            console,
            toast_center,
            // The plate starts at its floor: it grows when the user pulls
            // it, and it never takes the catalogue's rows to do it.
            nameplate_height: UiMetrics::NAMEPLATE_MIN_HEIGHT,
            nameplate_drag: None,
            scroll_handle: UniformListScrollHandle::new(),
            on_mutation: None,
            on_upgrade_all: None,
            on_retry_source: None,
            on_reload_installed: None,
            on_reload_updates: None,
            on_retry_details: None,
            copy_cmd_feedback: false,
            is_loading_pkgbuild: false,
            reduce_motion: config.reduce_motion,
            theme: config.theme,
            visual_style: config.visual_style,
            compact: config.compact,
            search_input,
            marker: None,
            view_marker: None,
            selection_epoch: 0,
            last_scope: initial_scope,
            scope_epoch: 0,
            scope_direction: 1.0,
            last_inspector_tab: InspectorTab::Overview,
            tab_direction: 1.0,
            scope_dial,
            _scope_dial_sub: dial_subscription,
            active_menu: None,
            menu_epoch: 0,
            state_btn_focus: cx.focus_handle(),
            sort_btn_focus: cx.focus_handle(),
            menu_surface_focus: cx.focus_handle(),
        }
    }

    pub fn toggle_menu(
        &mut self,
        target: Option<WorkbenchMenu>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match target {
            None => self.close_menu(window, cx),
            Some(menu) => {
                if let Some(ref current) = self.active_menu {
                    if current.menu == menu
                        && current.lifecycle != MenuLifecycle::Closing
                        && current.lifecycle != MenuLifecycle::Closed
                    {
                        self.close_menu(window, cx);
                        return;
                    }
                }
                self.open_menu(menu, window, cx);
            }
        }
    }

    pub fn open_menu(&mut self, menu: WorkbenchMenu, window: &mut Window, cx: &mut Context<Self>) {
        self.menu_epoch = self.menu_epoch.wrapping_add(1);
        let epoch = self.menu_epoch;

        if self.reduce_motion {
            self.active_menu = Some(ActiveMenuState {
                menu,
                lifecycle: MenuLifecycle::Open,
                anim_epoch: epoch,
                highlighted_index: 0,
            });
            window.focus(&self.menu_surface_focus, cx);
            cx.notify();
        } else {
            self.active_menu = Some(ActiveMenuState {
                menu,
                lifecycle: MenuLifecycle::Opening,
                anim_epoch: epoch,
                highlighted_index: 0,
            });
            window.focus(&self.menu_surface_focus, cx);
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(150))
                    .await;
                let _ = this.update(cx, |view, cx| {
                    if let Some(ref mut state) = view.active_menu {
                        if state.lifecycle == MenuLifecycle::Opening && state.anim_epoch == epoch {
                            state.lifecycle = MenuLifecycle::Open;
                            cx.notify();
                        }
                    }
                });
            })
            .detach();
        }
    }

    pub fn close_menu(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(current) = self.active_menu else {
            return;
        };

        if current.lifecycle == MenuLifecycle::Closing || current.lifecycle == MenuLifecycle::Closed
        {
            return;
        }

        match current.menu {
            WorkbenchMenu::State => window.focus(&self.state_btn_focus, cx),
            WorkbenchMenu::Sort => window.focus(&self.sort_btn_focus, cx),
        }

        if self.reduce_motion {
            self.active_menu = None;
            cx.notify();
        } else {
            self.menu_epoch = self.menu_epoch.wrapping_add(1);
            let epoch = self.menu_epoch;

            self.active_menu = Some(ActiveMenuState {
                menu: current.menu,
                lifecycle: MenuLifecycle::Closing,
                anim_epoch: epoch,
                highlighted_index: current.highlighted_index,
            });
            cx.notify();

            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let _ = this.update(cx, |view, cx| {
                    if let Some(ref state) = view.active_menu {
                        if state.lifecycle == MenuLifecycle::Closing && state.anim_epoch == epoch {
                            view.active_menu = None;
                            cx.notify();
                        }
                    }
                });
            })
            .detach();
        }
    }

    pub fn menu_item_count(&self, menu: WorkbenchMenu, scope: NavScope) -> usize {
        match menu {
            // Les états sont ceux du périmètre : le menu montre exactement la
            // liste que le clavier parcourt.
            WorkbenchMenu::State => PackageStateFilter::options_for(scope).len(),
            WorkbenchMenu::Sort => 6,
        }
    }

    pub fn navigate_menu(
        &mut self,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(current) = self.active_menu else {
            return;
        };

        let total = self.menu_item_count(current.menu, self.session.read(cx).scope);
        if total == 0 {
            return;
        }

        match key {
            "up" => {
                let next_idx = if current.highlighted_index == 0 {
                    total - 1
                } else {
                    current.highlighted_index - 1
                };
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = next_idx;
                }
                cx.notify();
            }
            "down" => {
                let next_idx = (current.highlighted_index + 1) % total;
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = next_idx;
                }
                cx.notify();
            }
            "home" => {
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = 0;
                }
                cx.notify();
            }
            "end" => {
                if let Some(ref mut st) = self.active_menu {
                    st.highlighted_index = total - 1;
                }
                cx.notify();
            }
            "enter" | "space" => {
                self.activate_menu_item(
                    current.menu,
                    current.highlighted_index,
                    window,
                    cx,
                );
            }
            _ => {}
        }
    }

    pub fn activate_menu_item(
        &mut self,
        menu: WorkbenchMenu,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match menu {
            WorkbenchMenu::State => {
                // Les états sont ceux du périmètre, dans l'ordre où le menu
                // les montre : les deux listes sont la même.
                let scope = self.session.read(cx).scope;
                if let Some(&state) = PackageStateFilter::options_for(scope).get(index) {
                    self.session.update(cx, |s, cx| {
                        s.set_state_filter(state, cx);
                    });
                    self.close_menu(window, cx);
                }
            }
            WorkbenchMenu::Sort => {
                let sorts = [
                    SortMode::Relevance,
                    SortMode::NameAsc,
                    SortMode::NameDesc,
                    SortMode::Source,
                    SortMode::InstalledFirst,
                    SortMode::UpdatesFirst,
                ];
                if let Some(&sort) = sorts.get(index) {
                    self.session.update(cx, |s, cx| {
                        s.set_sort_mode(sort, cx);
                    });
                    self.close_menu(window, cx);
                }
            }
        }
    }

    pub fn set_on_mutation(&mut self, handler: PackageMutationHandler) {
        self.on_mutation = Some(handler);
    }

    pub fn set_on_upgrade_all(&mut self, handler: SystemUpgradeHandler) {
        self.on_upgrade_all = Some(handler);
    }

    /// The edge is dragged: the band it shares with the catalogue is read
    /// from the drag itself (`DragMoveEvent::bounds`), where it is real —
    /// never from the window, which the drawer and the bank also share.
    pub fn on_edge_dragged(
        &mut self,
        pointer_y: f32,
        band_height: f32,
        cx: &mut Context<Self>,
    ) {
        if let Some(drag) = self.nameplate_drag {
            let current_y = pointer_y;
            let content_height = band_height;
            let new_height = nameplate_height_after_drag(
                drag.start_height,
                drag.start_pointer_y,
                current_y,
                content_height,
            );
            if (new_height - self.nameplate_height).abs() >= 1.0 {
                self.nameplate_height = new_height;
                cx.notify();
            }
        }
    }

    pub fn set_compact(&mut self, compact: bool, cx: &mut Context<Self>) {
        if self.compact != compact {
            self.compact = compact;
            cx.notify();
        }
    }

    pub fn set_reduce_motion(&mut self, reduce_motion: bool, cx: &mut Context<Self>) {
        if self.reduce_motion != reduce_motion {
            self.reduce_motion = reduce_motion;
            cx.notify();
        }
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        self.search_input
            .update(cx, |si, cx| si.set_theme(theme, cx));
        cx.notify();
    }

    pub fn set_visual_style(&mut self, style: VisualStyleId, cx: &mut Context<Self>) {
        self.visual_style = style;
        cx.notify();
    }

    pub fn on_pointer_up(&mut self, cx: &mut Context<Self>) {
        if self.nameplate_drag.is_some() {
            self.nameplate_drag = None;
            cx.notify();
        }
    }
}

impl Render for PackageWorkstationView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme;
        let entity = cx.entity().clone();
        let is_busy = self.console.read(cx).is_running();

        // La colonne de navigation : la largeur que l'utilisateur lui a
        // donnée, ou celle que la place lui laisse.
        let user_collapsed = self.session.read(cx).sidebar_collapsed;
        let sidebar_width = effective_sidebar_width(user_collapsed, window.viewport_size().width / px(1.0));
        let window_width = window.viewport_size().width / px(1.0);
        let content_width = content_width(window_width, sidebar_width);

        let active_list_width = content_width;

        let (
            scope,
            is_searching,
            selected_key,
            view_mode,
            active_tab,
            search_query,
            reduce_motion,
            inspector_tab_epoch,
        ) = {
            let s = self.session.read(cx);
            (
                s.scope,
                s.is_searching,
                s.selected_package_key.clone(),
                s.view_mode,
                s.inspector_tab,
                s.search_query.clone(),
                self.reduce_motion,
                s.inspector_tab_epoch,
            )
        };

        // Le périmètre a-t-il changé ? Le sens du changement donne la
        // direction de l'entrée : on ne coupe pas du même côté quand on
        // tourne le cadran vers la gauche ou vers la droite.
        if self.last_scope != scope {
            self.scope_direction = if scope_order(scope) >= scope_order(self.last_scope) {
                1.0
            } else {
                -1.0
            };
            self.last_scope = scope;
            self.scope_epoch += 1;
        }

        // L'onglet a-t-il changé ? Même règle : la coupe entre par le côté
        // où l'onglet est allé.
        if self.last_inspector_tab != active_tab {
            self.tab_direction = if active_tab.index() >= self.last_inspector_tab.index() {
                1.0
            } else {
                -1.0
            };
            self.last_inspector_tab = active_tab;
        }

        let (source_scope, state_filter, sort_mode) = {
            let s = self.session.read(cx);
            (s.source_scope, s.state_filter, s.sort_mode)
        };

        let raw_packages: std::sync::Arc<[UnifiedPackage]> = {
            let store = self.store.read(cx);
            match scope {
                NavScope::Results => std::sync::Arc::clone(&store.active_results),
                NavScope::Installed => std::sync::Arc::clone(&store.installed_packages),
                NavScope::Updates => std::sync::Arc::clone(&store.updates_packages),
            }
        };

        // La requête réduit le périmètre courant. Dans Résultats, les lignes
        // sont déjà celles de la recherche ; dans Installés et Mises à jour, les
        // lignes sont là et la requête les réduit.
        let query_narrows = scope != NavScope::Results && !search_query.trim().is_empty();

        let packages: std::sync::Arc<[UnifiedPackage]> = {
            if source_scope.is_all()
                && state_filter == PackageStateFilter::All
                && sort_mode == SortMode::Relevance
                && !query_narrows
            {
                raw_packages
            } else {
                let mut filtered = crate::state::query::visible_packages(
                    &raw_packages,
                    &source_scope,
                    state_filter,
                    sort_mode,
                );
                if query_narrows {
                    filtered.retain(|package| {
                        crate::state::query::matches_query(package, &search_query)
                    });
                }
                std::sync::Arc::from(filtered)
            }
        };

        let selected_pkg = selected_key
            .as_ref()
            .and_then(|key| packages.iter().find(|p| &p.key() == key).cloned());
        let selected_index = selected_key
            .as_ref()
            .and_then(|key| packages.iter().position(|p| &p.key() == key));

        // Le porteur : la barre qui voyage de la ligne quittée à la ligne
        // choisie. Elle n'existe qu'avec une sélection.
        let (marker, selection_epoch) = advance_marker(self.marker, selected_index);
        self.marker = marker;
        self.selection_epoch = selection_epoch;

        // Le même porteur, sur le commutateur de vue : sa marque va de la
        // cellule quittée à la cellule choisie.
        let (view_marker, _) = advance_marker(self.view_marker, Some(view_mode.index()));
        self.view_marker = view_marker;

        let entity_mode = entity.clone();
        let entity_select = entity.clone();
        let entity_split = entity.clone();
        let entity_move = entity.clone();
        let entity_up = entity.clone();


        // The query surface is in every scope: the filters act on every
        // scope, so the controls that change them are everywhere. The scope
        // decides what the controls offer, not whether they are there.
        let on_toggle_source = entity.clone();
        let on_state = entity.clone();
        let on_sort = entity.clone();
        let on_mode = entity_mode.clone();
        let on_menu = entity.clone();
        let on_nav_menu = entity.clone();
        let on_retry = entity.clone();
        let store = self.store.read(cx);

        // L'axe des sources : ce que chaque source tient, et combien de
        // lignes elle porte dans le périmètre courant.
        let source_axis = source_axis(scope, &packages, &source_scope);

        // La banque : sept ouvertures, une par source. Les sources sont un
        // axe, pas un filtre : la banque est toujours là.
        let source_bank = SourceBank::render(SourceBankProps {
            axis: &source_axis,
            source_health: &store.source_health,
            theme: &theme,
            epoch: scope.index(),
            reduce_motion: self.reduce_motion,
            on_toggle: {
                let entity = on_toggle_source.clone();
                Rc::new(move |kind, _w, cx| {
                    entity.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.toggle_source(kind, cx);
                        });
                    });
                })
            },
            on_retry: {
                let entity = on_retry.clone();
                Rc::new(move |kind, w, cx| {
                    let cb = entity.read(cx).on_retry_source.clone();
                    if let Some(cb) = cb {
                        cb(kind, w, cx);
                    }
                })
            },
        });

        let top_bar = QueryWorkbench::render(&QueryWorkbenchProps {
            search_input: self.search_input.clone(),
            list_pane_width: active_list_width,
            scope,
            state_filter,
            sort_mode,
            view_mode,
            view_marker: self.view_marker,
            active_menu_state: self.active_menu,
            reduce_motion: self.reduce_motion,
            theme: &theme,
            visual_style: self.visual_style,
            state_btn_focus: self.state_btn_focus.clone(),
            sort_btn_focus: self.sort_btn_focus.clone(),
            menu_surface_focus: self.menu_surface_focus.clone(),
            on_select_state: Rc::new(move |state, _w, cx| {
                on_state.update(cx, |view, cx| {
                    view.session.update(cx, |s, cx| {
                        s.set_state_filter(state, cx);
                    });
                });
            }),
            on_select_sort: Rc::new(move |sort, _w, cx| {
                on_sort.update(cx, |view, cx| {
                    view.session.update(cx, |s, cx| {
                        s.set_sort_mode(sort, cx);
                    });
                });
            }),
            on_select_view_mode: Rc::new(move |mode, _w, cx| {
                on_mode.update(cx, |view, cx| {
                    view.session.update(cx, |s, cx| {
                        s.set_view_mode(mode, cx);
                    });
                });
            }),
            on_toggle_menu: Rc::new(move |menu, window, cx| {
                on_menu.update(cx, |view, cx| {
                    view.toggle_menu(menu, window, cx);
                });
            }),
            on_navigate_menu: Rc::new(move |key, window, cx| {
                on_nav_menu.update(cx, |view, cx| {
                    view.navigate_menu(key, window, cx);
                });
            }),
            scope_action: match scope {
                NavScope::Updates => self.on_upgrade_all.clone().map(|handler| ScopeAction {
                    label: "Upgrade all",
                    handler,
                }),
                _ => None,
            },
        })
        .into_any_element();

        // ── 2. Corps de la liste (Cards vs Table) ───────────────────────────
        let (installed_err, updates_err) = {
            let st = self.store.read(cx);
            (st.installed_error.clone(), st.updates_error.clone())
        };

        let is_browse_empty =
            scope == NavScope::Results && search_query.trim().is_empty();

        let list_body = if let (NavScope::Installed, Some(err)) = (scope, installed_err)
        {
            let on_reload_installed_cb = self.on_reload_installed.clone();
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .p_8()
                .gap_4()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.danger)
                        .child(format!("Failed to load installed packages: {err}")),
                )
                .child(
                    div()
                        .id("retry_load_installed_btn")
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.accent)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.bg_app)
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.accent_hover))
                        .child("Retry")
                        .on_mouse_down(MouseButton::Left, move |_ev, w, cx| {
                            if let Some(ref cb) = on_reload_installed_cb {
                                cb(w, cx);
                            }
                        }),
                )
                .into_any_element()
        } else if let (NavScope::Updates, Some(err)) = (scope, updates_err) {
            let on_reload_updates_cb = self.on_reload_updates.clone();
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .p_8()
                .gap_4()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.danger)
                        .child(format!("Failed to check for updates: {err}")),
                )
                .child(
                    div()
                        .id("retry_load_updates_btn")
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.accent)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.bg_app)
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.accent_hover))
                        .child("Retry")
                        .on_mouse_down(MouseButton::Left, move |_ev, w, cx| {
                            if let Some(ref cb) = on_reload_updates_cb {
                                cb(w, cx);
                            }
                        }),
                )
                .into_any_element()
        } else if is_browse_empty {
            div()
                .flex_1()
                .child(UnifiedSearch::render_empty_discovery(&theme))
                .into_any_element()
        } else if packages.is_empty() {
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .size_full()
                .p_8()
                .text_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text_muted)
                        .child(if is_searching {
                            "Searching..."
                        } else {
                            "No matching packages found."
                        }),
                )
                .into_any_element()
        } else {
            let package_count = packages.len();
            let selected_name = selected_pkg.as_ref().map(|p| p.name.clone());
            let list_theme = theme;

            if view_mode == PackageViewMode::Table {
                div()
                    .debug_selector(|| "catalogue_table".to_string())
                    .flex()
                    .flex_col()
                    .size_full()
                    .child(
                        div().flex_1().h_full().overflow_hidden().child(
                            uniform_list("package-list-table", package_count, {
                                let packages = packages.clone();
                                let selected_name = selected_name.clone();
                                // The scale is shared: the bars of the readout
                                // are read against each other.
                                let scale_max = PackageTable::scale_max(&packages);
                                let is_compact = self.compact;

                                move |range, _window, _cx| {
                                    range
                                        .map(|idx| {
                                            let pkg = &packages[idx];
                                            let is_selected = selected_name
                                                .as_ref()
                                                .map(|n| n == &pkg.name)
                                                .unwrap_or(false);
                                            let pkg_key = pkg.key();
                                            let on_select = entity_select.clone();

                                            // The row, as the catalogue renders
                                            // it: the same component, the same
                                            // gesture.
                                            PackageTable::render_selectable_row(
                                                pkg,
                                                is_selected,
                                                &list_theme,
                                                is_compact,
                                                scale_max,
                                                Rc::new(move |_w, cx| {
                                                    let key = pkg_key.clone();
                                                    on_select.update(cx, |view, cx| {
                                                        view.session.update(cx, |s, cx| {
                                                            s.select_package(Some(key), cx);
                                                        });
                                                    });
                                                }),
                                            )
                                        })
                                        .collect()
                                }
                            })
                            .h_full()
                            // The handle the bench measures: the harness reads
                            // the bounds of the list under this name. A noop
                            // outside the tests.
                            .debug_selector(|| "package_list".to_string())
                            .track_scroll(&self.scroll_handle),
                        ),
                    )
                    .into_any_element()
            } else {
                let card_wrapper_h = if self.compact {
                    UiMetrics::CARD_WRAPPER_COMPACT
                } else {
                    UiMetrics::CARD_WRAPPER_NORMAL
                };
                let is_compact = self.compact;
                let visual_style = self.visual_style;
                uniform_list("package-list-surface", package_count, {
                    let packages = packages.clone();
                    let selected_name = selected_name.clone();

                    move |range, _window, _cx| {
                        range
                            .map(|idx| {
                                let pkg = &packages[idx];
                                let is_selected = selected_name
                                    .as_ref()
                                    .map(|n| n == &pkg.name)
                                    .unwrap_or(false);
                                let pkg_key = pkg.key();
                                let on_select = entity_select.clone();

                                div()
                                    .h(px(card_wrapper_h))
                                    .px_3()
                                    .py_1()
                                    .child(PackageCard::render(PackageCardProps {
                                        package: pkg,
                                        is_selected,
                                        theme: &list_theme,
                                        compact: is_compact,
                                        visual_style,
                                    }))
                                    .on_mouse_down(MouseButton::Left, move |_e, _w, cx| {
                                        let key = pkg_key.clone();
                                        on_select.update(cx, |view, cx| {
                                            view.session.update(cx, |s, cx| {
                                                s.select_package(Some(key), cx);
                                            });
                                        });
                                    })
                            })
                            .collect()
                    }
                })
                .h_full()
                .track_scroll(&self.scroll_handle)
                .into_any_element()
            }
        };

        // The list scrolls natively: the wheel moves it at once, and nothing
        // here re-renders itself frame by frame to ease it.
        let list_body = div().flex_1().overflow_hidden().child(list_body);

        // The readout: a recess cut into the plate.
        //
        // The frame stays when the readout is empty. The glass is calibrated —
        // the column names, the rules, the scale — and a calibrated glass that
        // waits reads as glass, not as a panel that failed to draw.
        // La barre du porteur : la sélection est un objet posé sur la ligne,
        // pas une bordure qui pousse le contenu. Elle voyage d'une ligne à
        // l'autre — et un voyage prend la courbe de l'aller-retour : la sortie
        // partirait si vite que la barre paraîtrait sauter d'une ligne à
        // l'autre, au lieu de s'y rendre.
        let row_height = if self.compact {
            UiMetrics::ROW_HEIGHT_COMPACT
        } else {
            UiMetrics::ROW_HEIGHT_NORMAL
        };
        let scroll_y: f32 = self.scroll_handle.0.borrow().base_handle.offset().y.into();
        let carrier = self.marker.map(|marker| {
            let row = marker.index as f32;
            let from = marker.from.map(|index| index as f32).unwrap_or(row);
            let bar = div()
                .id("readout_carrier")
                .absolute()
                .left(px(1.0))
                .top(px(carrier_top(
                    UiMetrics::ROW_HEIGHT_HEADER,
                    row_height,
                    row,
                    scroll_y,
                )))
                .w(px(3.0))
                .h(px(row_height))
                .bg(theme.accent);
            if self.reduce_motion {
                bar.into_any_element()
            } else {
                bar.with_animation(
                    ("readout_carrier", marker.epoch),
                    Animation::new(crate::state::MotionDurations::EMPHASIS)
                        .with_easing(crate::state::motion::ease_in_out),
                    move |bar, delta| {
                        let row =
                            crate::state::motion::carrier_travel(from, row, delta);
                        bar.top(px(carrier_top(
                            UiMetrics::ROW_HEIGHT_HEADER,
                            row_height,
                            row,
                            scroll_y,
                        )))
                    },
                )
                .into_any_element()
            }
        });

        // The ruler — the column names and the scale — is part of the recess.
        // It stays when the table is empty: a calibrated glass that waits
        // reads as glass, not as a panel that failed to draw. It does not stay
        // when there is no table yet: the discovery screen is a screen, not an
        // empty table.
        let shows_a_table = view_mode == PackageViewMode::Table && !is_browse_empty;
        let readout_recess = if view_mode == PackageViewMode::Table {
            div()
                .flex()
                .flex_col()
                .size_full()
                .flex_1()
                .overflow_hidden()
                .border_1()
                .border_color(theme.border)
                .bg(theme.bg_sidebar)
                .when(shows_a_table, |element| {
                    element.child(PackageTable::render_header(
                        &theme,
                        PackageTable::scale_max(&packages),
                    ))
                })
                .child(list_body)
                // Le porteur se pose sur les lignes, après elles : c'est un
                // objet de la sélection, pas une ligne du tableau.
                .children(carrier)
        } else {
            div()
                .flex()
                .flex_col()
                .size_full()
                .flex_1()
                .overflow_hidden()
                .border_1()
                .border_color(theme.border)
                .bg(theme.bg_sidebar)
                .child(list_body)
        };

        // Le périmètre entre par le côté où le cadran a tourné : l'entrée
        // n'est pas un fondu, c'est une coupe — le nouveau contenu se pose
        // décalé et vient à sa place.
        //
        // Le renfoncement est posé en absolu dans la bande : un décalage
        // relatif (`left`) ne déplace rien dans cette version de gpui — un
        // enfant en flux ignore son `inset` — et une entrée qui ne bouge pas
        // n'est pas une entrée. Pas d'opacité non plus : une couche d'opacité
        // autour d'une liste virtualisée se paie sur toute la fenêtre, et un
        // fondu n'est de toute façon pas une coupe.
        let readout_recess = if self.reduce_motion || self.scope_epoch == 0 {
            readout_recess.into_any_element()
        } else {
            let direction = self.scope_direction;
            readout_recess.with_animation(
                ("scope_entry", self.scope_epoch),
                Animation::new(crate::state::MotionDurations::SCOPE_ENTRY)
                    .with_easing(crate::state::motion::ease_out),
                move |element, delta| {
                    element.left(px(direction * crate::state::motion::cut_travel(
                        delta,
                        SCOPE_ENTRY_TRAVEL,
                    )))
                },
            )
            .into_any_element()
        };

        // La bande du renfoncement : une scène qui donne au renfoncement sa
        // place et sa coupe. Le renfoncement lui-même est en absolu, et c'est
        // son décalage qui porte l'entrée.
        let readout_recess = div()
            .relative()
            .w_full()
            .flex_1()
            .min_h(px(0.0))
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .size_full()
                    .child(readout_recess),
            );

        // The readout: the band where the work happens, in full width.
        let readout = div()
            .id("workstation_readout")
            // The handle the audit measures: the harness reads the bounds of
            // the band under this name. A noop outside the tests.
            .debug_selector(|| "workstation_readout".to_string())
            .flex()
            .flex_col()
            .w_full()
            .flex_1()
            // The catalogue keeps its rows: the plate yields first.
            .min_h(px(UiMetrics::CATALOGUE_MIN_HEIGHT))
            .overflow_hidden()
            .child(top_bar)
            .child(readout_recess);

        // ── 3. The nameplate's edge, drawn between the bands ────────────────
        //
        // In the flow, the edge separates the two bands. Over the catalogue,
        // it rides on the sheet's top edge: the same gesture, but it lifts
        // the sheet instead of resizing it.
        let is_resizing = self.nameplate_drag.is_some();
        let edge_style = |style: gpui::Div| {
            style
                .id("workstation_nameplate_edge")
                .w_full()
                .h(px(UiMetrics::SPLITTER_THICKNESS))
                .bg(if is_resizing {
                    theme.accent
                } else {
                    theme.border
                })
                .cursor_row_resize()
                .hover({
                    let h = theme.accent;
                    move |s| s.bg(h)
                })
                .on_drag(NameplateEdge, |_, _, _, cx| cx.new(|_| NameplateEdge))
                .on_mouse_down(MouseButton::Left, move |event, _window, cx| {
                    // The press gives where the drag starts; only the
                    // distance travelled matters (`on_edge_dragged`).
                    let pointer_y = event.position.y.to_f64() as f32;
                    entity_split.update(cx, |view, cx| {
                        view.nameplate_drag = Some(NameplateDragState {
                            start_height: view.nameplate_height,
                            start_pointer_y: pointer_y,
                        });
                        cx.notify();
                    });
                })
        };
        let splitter = edge_style(div().flex_shrink_0());

        // ── 4. Panneau d'inspection sémantique ──────────────────────────────
        let alpm_details = selected_key
            .as_ref()
            .and_then(|k| self.store.read(cx).get_cached_details(k).cloned());

        // Le même nom, dans les autres sources. La fiche le dit : un nom
        // n'est pas un paquet, et les sources ne sont pas interchangeables.
        let also_in: Vec<AlsoIn> = match selected_pkg.as_ref() {
            Some(package) => {
                let store = self.store.read(cx);
                also_in(package, &store.active_results)
            }
            None => Vec::new(),
        };
        let detail_error = selected_key
            .as_ref()
            .and_then(|k| self.store.read(cx).get_detail_error(k).cloned());

        let cached_pkgbuild = selected_pkg.as_ref().and_then(|p| {
            if p.source_type == "AUR" {
                self.store.read(cx).get_cached_pkgbuild(&p.name)
            } else {
                None
            }
        });

        let on_retry_details = if detail_error.is_some() {
            if let Some(key) = selected_key.clone() {
                self.on_retry_details.clone().map(|cb| {
                    Rc::new(move |w: &mut Window, cx: &mut App| {
                        cb(&key, w, cx);
                    }) as Rc<dyn Fn(&mut Window, &mut App)>
                })
            } else {
                None
            }
        } else {
            None
        };

        let selected_pkg_clone = selected_pkg.clone();
        let on_mutation_cb = self.on_mutation.clone();
        let on_mutation_cb_rm = self.on_mutation.clone();
        let entity_tab = entity.clone();
        let entity_nav_dep = entity.clone();
        let tc_entity = self.toast_center.clone();

        // La plaque au repos : le canal que la banque met en avant, ce qu'il
        // porte, et le nombre de lignes du périmètre. Une inscription, pas une
        // excuse.
        let anchor_entry = source_axis.get(crate::state::anchor_index(&source_axis));
        let quiet = QuietPlate {
            heading: "THE BANK",
            subject: anchor_entry
                .map(|entry| entry.label.to_string())
                .unwrap_or_else(|| "All sources".to_string()),
            detail: anchor_entry.map_or_else(String::new, |entry| {
                let rows = if entry.count == 1 {
                    "one row in this scope".to_string()
                } else {
                    format!("{} rows in this scope", entry.count)
                };
                format!("{} · {}", entry.holds, rows)
            }),
        };

        let inspector_element = PackageInspectorView::render_with_motion(
            PackageInspectorProps {
                package: selected_pkg.as_ref(),
                quiet,
                alpm_details: alpm_details.as_ref(),
                also_in: &also_in,
                detail_error: detail_error.as_deref(),
                pkgbuild: cached_pkgbuild,
                is_loading_pkgbuild: self.is_loading_pkgbuild,
                active_tab,
                theme: &theme,
                is_busy,
                copy_feedback: self.copy_cmd_feedback,
                on_select_tab: Rc::new(move |tab, _w, cx| {
                    entity_tab.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.set_inspector_tab(tab, cx);
                        });
                    });
                }),
                on_install: selected_pkg_clone.as_ref().and_then(|p| {
                    on_mutation_cb.as_ref().map(|cb| {
                        let p_c = p.clone();
                        let cb_c = cb.clone();
                        Rc::new(move |window: &mut Window, cx: &mut App| {
                            cb_c(&p_c, true, window, cx);
                        }) as Rc<dyn Fn(&mut Window, &mut App)>
                    })
                }),
                on_remove: selected_pkg_clone.as_ref().and_then(|p| {
                    on_mutation_cb_rm.as_ref().map(|cb| {
                        let p_c = p.clone();
                        let cb_c = cb.clone();
                        Rc::new(move |window: &mut Window, cx: &mut App| {
                            cb_c(&p_c, false, window, cx);
                        }) as Rc<dyn Fn(&mut Window, &mut App)>
                    })
                }),
                on_copy_install_cmd: selected_pkg_clone.as_ref().and_then(|p| {
                    canonical_install_command(p).map(|cmd| {
                        let tc = tc_entity.clone();
                        let rm = reduce_motion;
                        Rc::new(move |_w: &mut Window, cx: &mut App| {
                            cx.write_to_clipboard(ClipboardItem::new_string(cmd.clone()));
                            tc.update(cx, |center, cx| {
                                center.post(
                                    ToastKind::Success,
                                    "Command copied",
                                    cmd.clone(),
                                    None,
                                    rm,
                                    cx,
                                );
                            });
                        }) as Rc<dyn Fn(&mut Window, &mut App)>
                    })
                }),
                on_copy_pkgbuild: cached_pkgbuild.cloned().map(|content| {
                    let tc = tc_entity.clone();
                    let rm = reduce_motion;
                    Rc::new(move |_w: &mut Window, cx: &mut App| {
                        cx.write_to_clipboard(ClipboardItem::new_string(content.clone()));
                        tc.update(cx, |center, cx| {
                            center.post(
                                ToastKind::Success,
                                "PKGBUILD copied",
                                "The PKGBUILD recipe has been copied to your clipboard.",
                                None,
                                rm,
                                cx,
                            );
                        });
                    }) as Rc<dyn Fn(&mut Window, &mut App)>
                }),
                on_navigate_package: Some(Rc::new(move |pkg_name, _w, cx| {
                    entity_nav_dep.update(cx, |view, cx| {
                        view.session.update(cx, |s, cx| {
                            s.set_scope(NavScope::Results, cx);
                            s.set_source_scope(SourceScope::all(), cx);
                            s.set_state_filter(PackageStateFilter::All, cx);
                            s.set_search_query(pkg_name.clone(), cx);
                            s.select_package(None, cx);
                        });
                    });
                })),
                on_retry_details,
            },
            reduce_motion,
            inspector_tab_epoch,
            self.tab_direction,
        );

        // La plaque entre avec la sélection : elle ne se remplace pas d'un
        // coup, elle arrive. Une coupe, pas un fondu : c'est la position qui
        // donne le geste.
        let inspector_element = if self.reduce_motion {
            inspector_element
        } else {
            // La plaque remplit la bande que le flex lui a donnée.
            div()
                .absolute()
                .top(px(0.0))
                .left(px(0.0))
                .size_full()
                .child(inspector_element)
                .with_animation(
                    ("nameplate_entry", self.selection_epoch),
                    Animation::new(crate::state::MotionDurations::PLATE_ENTRY)
                        .with_easing(crate::state::motion::ease_out),
                    |element, delta| {
                        element.top(px(crate::state::motion::cut_travel(
                            delta,
                            PLATE_ENTRY_TRAVEL,
                        )))
                    },
                )
                .into_any_element()
        };

        // The nameplate: the caption of the readout, in full width, under it.
        //
        // It is a band under the catalogue, and it yields before the
        // catalogue is reduced: one always knows which package is selected.
        // The flex decides, where the room is real: the plate asks for the
        // height the user gave it, shrinks first (the catalogue has a floor),
        // never below its own floor and never above its share of the band.
        let nameplate = div()
            .id("workstation_nameplate")
            // The handle the audit measures: the harness reads the bounds
            // of the band under this name. A noop outside the tests.
            .debug_selector(|| "workstation_nameplate".to_string())
            .w_full()
            .h(px(self.nameplate_height))
            .flex_shrink(1.0)
            .min_h(px(UiMetrics::NAMEPLATE_MIN_HEIGHT))
            .max_h(relative(UiMetrics::NAMEPLATE_MAX_RATIO))
            .overflow_hidden()
            .child(inspector_element)
            .into_any_element();

        // ── 5. Conteneur racine de la station de travail absorbant les moves ──
        div()
            .id("package_workstation_root")
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .overflow_hidden()
            .on_mouse_up(MouseButton::Left, move |_event, _window, cx| {
                entity_up.update(cx, |view, cx| {
                    view.on_pointer_up(cx);
                });
            })
            .child(source_bank)
            .child(
                // The band the catalogue and the plate share. The edge's drag
                // reads its bounds from here, where they are real.
                div()
                    .id("workstation_band")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .overflow_hidden()
                    .on_drag_move::<NameplateEdge>(move |event, _window, cx| {
                        let band_height: f32 = event.bounds.size.height.into();
                        let pointer_y: f32 = event.event.position.y.into();
                        entity_move.update(cx, |view, cx| {
                            view.on_edge_dragged(pointer_y, band_height, cx);
                        });
                    })
                    .child(readout)
                    .child(splitter)
                    .child(nameplate),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    /// L'échelle horizontale : la colonne se replie quand le catalogue n'a
    /// plus la place, et elle reprend sa forme quand la fenêtre la rend.
    ///
    /// C'est la règle de l'échelle du contrat spatial, en exécutable : la
    /// colonne de navigation est la première surface qui peut rétrécir, et
    /// elle ne disparaît jamais.
    #[test]
    fn the_spine_folds_when_the_catalogue_needs_the_room() {
        // Une fenêtre large : la colonne garde la forme de l'utilisateur.
        assert!(!spine_is_forced_collapsed(1440.0));
        assert_eq!(effective_sidebar_width(false, 1440.0), UiMetrics::SIDEBAR_EXPANDED);

        // Au plancher de la fenêtre, la colonne se replie : le catalogue
        // garde sa place.
        assert!(
            spine_is_forced_collapsed(720.0),
            "à 720 px, la colonne dépliée ne laisserait que {} px au catalogue",
            content_width(720.0, UiMetrics::SIDEBAR_EXPANDED)
        );
        assert_eq!(effective_sidebar_width(false, 720.0), UiMetrics::SIDEBAR_COLLAPSED);

        // La colonne repliée laisse au catalogue son plancher.
        assert!(
            content_width(720.0, UiMetrics::SIDEBAR_COLLAPSED) >= UiMetrics::CATALOGUE_MIN_WIDTH,
            "le catalogue garde {} px",
            content_width(720.0, UiMetrics::SIDEBAR_COLLAPSED)
        );

        // La forme que l'utilisateur a donnée reste la sienne : la règle
        // ne fait que la plier.
        assert_eq!(effective_sidebar_width(true, 1440.0), UiMetrics::SIDEBAR_COLLAPSED);

        // Le cas qui échoue, et il est gardé : un compositeur qui accorde
        // moins que le plancher. À 600 px, la colonne repliée laisse 544 px
        // au catalogue — 116 px de moins que son plancher qualifié (660,
        // mesuré). La composition ne peut pas tenir sa promesse là, et le
        // banc le dit au lieu de le cacher.
        let constrained = 600.0;
        assert!(spine_is_forced_collapsed(constrained));
        let content = content_width(constrained, UiMetrics::SIDEBAR_COLLAPSED);
        assert_eq!(content, 544.0);
        assert!(
            content < UiMetrics::CATALOGUE_MIN_WIDTH,
            "sous le plancher du compositeur, le catalogue est sous son minimum : {content} contre {}",
            UiMetrics::CATALOGUE_MIN_WIDTH
        );
    }

    /// Le plancher de la fenêtre, recompté : il ne reste rien.
    ///
    /// Le budget du plancher est une somme, et la somme s'additionne : la
    /// ligne du catalogue (495), le nom qu'elle porte (22 + 143), et la
    /// colonne repliée (56). C'est 716, et la fenêtre en demande 720.
    /// L'écart de 104 px que l'ancienne justification ne savait pas
    /// expliquer est donc le nom du paquet : la place que la ligne ne
    /// Donnerait pas au texte, et que le texte prend.
    #[test]
    fn the_window_floor_budget_adds_up() {
        // The catalogue's qualified minimum: 560 px, measured on the real
        // row with the ladder — the name keeps its floor, and the columns
        // that say *where* give way.
        let catalogue = 560.0_f32;
        let spine = UiMetrics::SIDEBAR_COLLAPSED;

        assert_eq!(
            catalogue, UiMetrics::CATALOGUE_MIN_WIDTH,
            "the catalogue's qualified minimum"
        );
        assert_eq!(
            catalogue + spine,
            616.0,
            "the budget of the floor, before the rounding"
        );
        assert_eq!(
            crate::config::MIN_WINDOW_WIDTH,
            620.0,
            "and the window asks for 620 — the budget, rounded up"
        );
        assert!(
            crate::config::MIN_WINDOW_WIDTH >= catalogue + spine,
            "the floor carries the whole budget"
        );
        // And at the floor, the catalogue keeps its minimum.
        let content = content_width(
            crate::config::MIN_WINDOW_WIDTH,
            UiMetrics::SIDEBAR_COLLAPSED,
        );
        assert!(
            content >= UiMetrics::CATALOGUE_MIN_WIDTH,
            "au plancher, le catalogue garde {content} px pour un minimum de {}",
            UiMetrics::CATALOGUE_MIN_WIDTH
        );
    }

    /// Le plancher de la fenêtre est dérivé, pas choisi.
    ///
    /// Le catalogue garde 560 px, la colonne repliée 56 : le plancher est
    /// leur somme. Une fenêtre plus étroite ne tiendrait pas la composition,
    /// et le compositeur peut toujours en accorder une plus étroite — la
    /// composition, elle, doit savoir tenir.
    #[test]
    fn the_window_floor_is_derived_from_the_catalogue_and_the_spine() {
        let floor = UiMetrics::CATALOGUE_MIN_WIDTH + UiMetrics::SIDEBAR_COLLAPSED;
        assert!(
            crate::config::MIN_WINDOW_WIDTH >= floor,
            "le plancher ({}) doit porter le catalogue et la colonne repliée ({})",
            crate::config::MIN_WINDOW_WIDTH,
            floor
        );
        // Et il ne sert à rien d'être plus large que ce que la composition
        // demande : la colonne repliée, c'est déjà la règle.
        assert!(
            crate::config::MIN_WINDOW_WIDTH <= UiMetrics::CATALOGUE_MIN_WIDTH
                + UiMetrics::SIDEBAR_EXPANDED,
            "au-delà, la colonne n'a plus besoin de se replier : le plancher serait du comfort"
        );
    }

    #[test]
    fn the_content_width_follows_the_mapped_window() {
        // La largeur mappée moins le rail : c'est elle qui décide du partage
        // entre la liste et l'inspecteur.
        assert_eq!(
            content_width(1056.0, UiMetrics::SIDEBAR_EXPANDED),
            866.0
        );
        assert_eq!(
            content_width(1056.0, UiMetrics::SIDEBAR_COLLAPSED),
            1000.0
        );

        // Un rail plus large que la fenêtre ne donne pas une largeur négative.
        assert_eq!(content_width(100.0, UiMetrics::SIDEBAR_EXPANDED), 0.0);
    }

    #[test]
    fn the_nameplate_stays_inside_its_bounds_at_the_heights_a_tiling_compositor_uses() {
        // Les hauteurs que le compositeur à tuiles a données pendant la
        // mesure : 720px et 1440px. Dans les deux, la plaque de nom reste
        // entre son plancher et sa part — elle n'avale jamais la lecture.
        for window_height in [720.0_f32, 1440.0] {
            let content = (window_height - UiMetrics::NAMEPLATE_MIN_HEIGHT).max(0.0);
            let nameplate = nameplate_height(content, 320.0);

            assert!(
                nameplate >= UiMetrics::NAMEPLATE_MIN_HEIGHT - 0.01,
                "la plaque de nom tombe sous son plancher : {nameplate}px dans une fenêtre de {window_height}px"
            );
            assert!(
                nameplate <= content * UiMetrics::NAMEPLATE_MAX_RATIO + 0.01,
                "la plaque de nom dépasse sa part : {nameplate}px dans une fenêtre de {window_height}px"
            );
            // Et la lecture garde le reste.
            assert!(
                content - nameplate > 0.0,
                "la lecture garde de la place : {content}px de contenu, {nameplate}px de plaque"
            );
        }
    }

    #[test]
    fn the_nameplate_is_pulled_up_and_grows() {
        // On tire vers le haut : la plaque grandit.
        let grown = nameplate_height_after_drag(300.0, 500.0, 450.0, 900.0);
        assert_eq!(grown, 350.0);
        // On tire vers le bas : elle rétrécit.
        let shrunk = nameplate_height_after_drag(300.0, 500.0, 550.0, 900.0);
        assert_eq!(shrunk, 250.0);
    }

    #[test]
    fn the_nameplate_never_swallows_the_readout() {
        // Tirée vers le haut sans fin, elle s'arrête à sa part.
        let content = 900.0;
        let pulled = nameplate_height_after_drag(300.0, 800.0, 0.0, content);
        assert_eq!(pulled, content * UiMetrics::NAMEPLATE_MAX_RATIO);
        // Tirée vers le bas sans fin, elle s'arrête à son plancher.
        let pushed = nameplate_height_after_drag(300.0, 0.0, 2000.0, content);
        assert_eq!(pushed, UiMetrics::NAMEPLATE_MIN_HEIGHT);
    }

    #[test]
    fn the_nameplate_is_bounded_in_a_short_window() {
        // Une bande basse : il n'y a pas de place pour que la plaque grandisse
        // — elle reste à son plancher, et le catalogue garde ce qui reste.
        let band = 200.0;
        let (min, max) = nameplate_bounds(band);
        assert_eq!(min, UiMetrics::NAMEPLATE_MIN_HEIGHT);
        assert_eq!(max, UiMetrics::NAMEPLATE_MIN_HEIGHT);
        assert_eq!(
            nameplate_height(band, 1000.0),
            min,
            "la plaque ne grandit pas dans une bande trop basse"
        );

        // Une bande large : la plaque s'arrête à sa part, même tirée sans
        // fin, et le catalogue garde ses lignes.
        let band = 600.0;
        let (min, max) = nameplate_bounds(band);
        assert!(
            max > min,
            "dans une bande large, la plaque a de la place : {max}"
        );
        assert!(
            max <= band * UiMetrics::NAMEPLATE_MAX_RATIO + 0.01,
            "sans jamais prendre plus que sa part de la bande : {max}"
        );
        assert_eq!(
            nameplate_height(band, 1000.0),
            max,
            "la plaque s'arrête à sa part, même tirée sans fin"
        );
        // Et le catalogue garde le minimum, même quand la plaque prend le
        // reste.
        let catalogue = band
            - UiMetrics::SPLITTER_THICKNESS
            - nameplate_height(band, 1000.0);
        assert!(
            catalogue >= UiMetrics::CATALOGUE_MIN_HEIGHT - 0.01,
            "le catalogue garde ses lignes : {catalogue}"
        );
    }

    #[test]
    fn the_carrier_sits_on_the_line_it_marks() {
        // Le haut du renfoncement, puis les lignes, moins le défilement.
        let header = UiMetrics::ROW_HEIGHT_HEADER;
        let row = UiMetrics::ROW_HEIGHT_NORMAL;
        assert_eq!(carrier_top(header, row, 0.0, 0.0), header);
        assert_eq!(carrier_top(header, row, 3.0, 0.0), header + 3.0 * row);
        assert_eq!(
            carrier_top(header, row, 3.0, row),
            header + 2.0 * row,
            "une ligne défilée remonte d'une ligne"
        );
        // Entre deux lignes, quand elle voyage.
        assert_eq!(
            carrier_top(header, row, 1.5, 0.0),
            header + 1.5 * row
        );
    }

    #[test]
    fn the_scope_dirties_the_side_of_the_cut() {
        // Le cadran tourne : la gauche, le haut, la droite. Deux périmètres
        // voisins ne coupent pas du même côté quand on inverse le sens.
        assert!(scope_order(NavScope::Results) < scope_order(NavScope::Installed));
        assert!(scope_order(NavScope::Installed) < scope_order(NavScope::Updates));
    }

    /// La composition, mesurée : ce que la fenêtre donne, ce que les bandes
    /// prennent, et ce qui reste au catalogue.
    ///
    /// C'est l'audit du contrat spatial (CONTRACT-01) rendu exécutable : on
    /// dessine la vraie station — la banque, la lecture, la plaque — dans
    /// une fenêtre réelle, et on lit les hauteurs que la fenêtre a
    /// dessinées.
    ///
    /// La station ne mesure plus la fenêtre : le propriétaire de la mise en
    /// page lui attribue sa bande, et c'est **cette** bande qui décide de la
    /// part de la plaque. Le second dessin est ce cas : la station reçoit
    /// 372 px — ce que le tiroir de journal lui laisse — et la plaque se
    /// règle sur cette mesure. Avant, elle se réglait sur la fenêtre, et le
    /// catalogue tombait à 6,5 px pendant que sa légende gardait 260.
    #[gpui::test]
    fn the_composition_is_measured_in_a_real_window(cx: &mut gpui::TestAppContext) {
        use gpui::prelude::*;
        use gpui::{div, point, px, size, Context, Render, Window};

        struct Fixture {
            station: Entity<PackageWorkstationView>,
            band: Option<f32>,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let station = self.station.clone();
                let content = div().size_full().child(station).into_any_element();
                match self.band {
                    Some(height) => div()
                        .w_full()
                        .h(px(height))
                        .overflow_hidden()
                        .child(content)
                        .into_any_element(),
                    None => content,
                }
            }
        }

        let session = cx.new(|_| AppSession::new());
        let store = cx.new(|_| {
            PackageStore::new(crate::backend::client::CalixClient::default())
        });
        let console = cx.new(|_| ConsoleModel::new());
        let toast_center = cx.new(|_| ToastCenter::new());
        let station = cx.new(|cx| {
            PackageWorkstationView::new(
                session,
                store,
                console,
                toast_center,
                PackageWorkstationConfig {
                    theme: Theme::dark(),
                    reduce_motion: false,
                    compact: false,
                    visual_style: VisualStyleId::default(),
                },
                cx,
            )
        });

        let window = (px(1024.0), px(680.0));
        let cx = cx.add_empty_window();
        let fixture = cx.new(|_| Fixture {
            station: station.clone(),
            band: None,
        });


        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.0), px(0.0)),
                size(window.0, window.1),
                |_window, _cx| fixture.clone().into_element(),
            );
        };

        // ── The whole window: the composition at the window's floor.
        draw(cx);
        let bank = cx.debug_bounds("source_bank").expect("the bank is drawn");
        let readout = cx
            .debug_bounds("workstation_readout")
            .expect("the readout is drawn");
        let nameplate = cx
            .debug_bounds("workstation_nameplate")
            .expect("the nameplate is drawn");

        // The three bands fill the window: nothing is lost between them.
        assert!(
            (bank.size.height + readout.size.height + nameplate.size.height).floor()
                <= window.1.floor(),
            "the bands fit in the window: {} + {} + {} <= {}",
            bank.size.height,
            readout.size.height,
            nameplate.size.height,
            window.1
        );
        assert!(
            readout.size.height > px(0.0),
            "the catalogue has room when the window is whole: {}",
            readout.size.height
        );

        // ── The station in the band the log drawer leaves it.
        //
        // 372 px: the band cannot hold the bank, the edge, the catalogue's
        // six rows *and* the plate's chrome. The plate yields first — it
        // keeps its identity (48 px) and the catalogue keeps the rest.
        let band_height = 372.0;
        fixture.update(cx, |fixture, _cx| fixture.band = Some(band_height));
        draw(cx);
        let squeezed_readout = cx
            .debug_bounds("workstation_readout")
            .expect("the readout is drawn");
        // The bank takes what its content needs: read, never assumed.
        let squeezed_bank = cx.debug_bounds("source_bank").expect("the bank is drawn");

        // The plate stays in the flow — it yields, it does not leave: one
        // always knows which package is selected.
        assert!(
            cx.debug_bounds("workstation_nameplate").is_some(),
            "la plaque reste une bande du flux"
        );

        // And the catalogue takes the band below the bank, less the plate's
        // floor: the plate yields, it does not leave, and the catalogue
        // keeps its floor.
        assert_eq!(
            squeezed_readout.size.height,
            px(band_height
                - f32::from(squeezed_bank.size.height)
                - UiMetrics::SPLITTER_THICKNESS
                - UiMetrics::NAMEPLATE_MIN_HEIGHT),
            "le catalogue garde son plancher, la plaque son identité : {}",
            squeezed_readout.size.height
        );
        assert!(
            squeezed_readout.size.height >= px(UiMetrics::CATALOGUE_MIN_HEIGHT) - px(0.5),
            "et il garde son plancher de six lignes : {}",
            squeezed_readout.size.height
        );

        // In the flow, the plate takes its chrome from the band, and the
        // catalogue keeps the rest.
        assert_eq!(
            readout.size.height,
            px(window.1 / px(1.0)
                - f32::from(bank.size.height)
                - UiMetrics::SPLITTER_THICKNESS
                - UiMetrics::NAMEPLATE_MIN_HEIGHT),
            "dans le flux, la plaque prend son chrome et le catalogue le reste : {}",
            readout.size.height
        );
    }

    /// La porte de l'interaction : un clic sur une ligne du catalogue
    /// sélectionne ce paquet.
    ///
    /// C'est le corps — la surface de priorité 2. Un clic qui ne sélectionne
    /// rien est un catalogue mort, et la porte le dit.
    ///
    /// Ici, la ligne est dans la **vraie** liste, avec la vraie station
    /// autour d'elle : le clic passe par le composant, la sélection va à la
    /// session, et la session est ce que l'on vérifie.
    #[gpui::test]
    fn a_click_on_a_row_selects_that_package(cx: &mut gpui::TestAppContext) {
        use crate::backend::models::{AlpmPackage, UnifiedPackageSource};
        use gpui::prelude::*;
        use gpui::{point, px, size, Context, Render, Window};

        struct Fixture {
            station: Entity<PackageWorkstationView>,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let station = self.station.clone();
                // A *definite* height, not a percentage: the bench's measured
                // layout path does not resolve a percentage against a
                // percentage.
                div().h(px(700.0)).w(px(1080.0)).child(station)
            }
        }

        let session = cx.new(|_| AppSession::new());
        let store = cx.new(|_| {
            PackageStore::new(crate::backend::client::CalixClient::default())
        });
        let console = cx.new(|_| ConsoleModel::new());
        let toast_center = cx.new(|_| ToastCenter::new());

        // Two rows, so the click has to pick *one* and not merely find one.
        let packages: Vec<UnifiedPackage> = ["ripgrep", "7zip"]
            .iter()
            .map(|name| UnifiedPackage {
                name: (*name).to_string(),
                version: "14.1.0-1".to_string(),
                description: "a package".to_string(),
                source_type: "ALPM".to_string(),
                repository_or_remote: "extra".to_string(),
                is_installed: true,
                has_update: false,
                new_version: None,
                inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
            })
            .collect();
        store.update(cx, |store, _cx| {
            store.installed_packages = std::sync::Arc::from(packages);
        });
        // The installed scope — the default is the results, which are empty
        // here — and the table, not the cards: the catalogue's list is what
        // carries the rows.
        session.update(cx, |session, cx| {
            session.set_scope(NavScope::Installed, cx);
            session.set_view_mode(PackageViewMode::Table, cx);
        });

        let station = cx.new(|cx| {
            PackageWorkstationView::new(
                session.clone(),
                store.clone(),
                console.clone(),
                toast_center.clone(),
                PackageWorkstationConfig {
                    theme: Theme::dark(),
                    reduce_motion: false,
                    compact: false,
                    visual_style: VisualStyleId::default(),
                },
                cx,
            )
        });

        let cx = cx.add_empty_window();
        let fixture = cx.new(|_| Fixture {
            station: station.clone(),
        });

        // The row the list drew — *named*, so that the click lands on the
        // package the test means and not on the first one of the list.
        cx.draw(
            point(px(0.0), px(0.0)),
            size(px(1080.0), px(700.0)),
            |_window, _cx| fixture.clone().into_element(),
        );
        let row = cx
            .debug_bounds("package_row-ripgrep")
            .expect("the list draws its rows");
        assert!(row.size.height > px(0.0), "the row has a height");

        // The click.
        cx.simulate_click(
            point(
                row.origin.x + row.size.width / 2.0,
                row.origin.y + row.size.height / 2.0,
            ),
            gpui::Modifiers::default(),
        );

        // And the selection.
        let selected = cx.read(|app| session.read(app).selected_package_key.clone());
        assert_eq!(
            selected.map(|key| key.name),
            Some("ripgrep".to_string()),
            "le clic sur la ligne a sélectionné ce paquet"
        );
    }

    /// La composition au plancher de la fenêtre, mesurée.
    ///
    /// 620 × 480 : la bande que l'espace de travail laisse à la station est
    /// 480 − 88 (l'en-tête) − 36 (le bandeau du tiroir) = 356 px. La question
    /// est alors une seule : le catalogue garde-t-il ses six lignes, et la
    /// plaque son identité ?
    #[gpui::test]
    fn the_composition_at_the_window_floor_is_measured(cx: &mut gpui::TestAppContext) {
        use gpui::prelude::*;
        use gpui::{div, point, px, size, Context, Render, Window};

        struct Fixture {
            station: Entity<PackageWorkstationView>,
            band: f32,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let station = self.station.clone();
                // The band the station really gets: the window's height, less
                // the header's band and the drawer's.
                div().h(px(self.band)).w_full().child(station)
            }
        }

        let session = cx.new(|_| AppSession::new());
        let store = cx.new(|_| PackageStore::new(crate::backend::client::CalixClient::default()));
        let console = cx.new(|_| ConsoleModel::new());
        let toast_center = cx.new(|_| ToastCenter::new());
        let station = cx.new(|cx| {
            PackageWorkstationView::new(
                session.clone(),
                store.clone(),
                console.clone(),
                toast_center.clone(),
                PackageWorkstationConfig {
                    theme: Theme::dark(),
                    reduce_motion: false,
                    compact: false,
                    visual_style: VisualStyleId::default(),
                },
                cx,
            )
        });

        let window = (px(620.0), px(520.0));
        let cx = cx.add_empty_window();
        let band = 520.0 - UiMetrics::CONTEXT_HEADER_HEIGHT - UiMetrics::DRAWER_HEADER_HEIGHT;
        let fixture = cx.new(|_| Fixture {
            station: station.clone(),
            band,
        });
        cx.draw(
            point(px(0.0), px(0.0)),
            size(window.0, window.1),
            |_window, _cx| fixture.clone().into_element(),
        );

        let readout = cx
            .debug_bounds("workstation_readout")
            .expect("the readout is drawn");
        let plate = cx
            .debug_bounds("workstation_nameplate")
            .expect("the plate is drawn");
        let catalogue: f32 = readout.size.height.into();
        let plate_height: f32 = plate.size.height.into();

        eprintln!(
            "PROBE at the floor: the catalogue {catalogue} px ({} rows), the plate {plate_height} px",
            (catalogue / UiMetrics::ROW_HEIGHT_NORMAL) as i32
        );

        // The catalogue keeps its floor: six rows.
        assert!(
            catalogue >= UiMetrics::CATALOGUE_MIN_HEIGHT - 0.5,
            "le catalogue garde son plancher : {catalogue} pour {}",
            UiMetrics::CATALOGUE_MIN_HEIGHT
        );
        // And the plate keeps its identity: it is still there.
        assert!(
            plate_height >= UiMetrics::NAMEPLATE_MIN_HEIGHT - 0.5,
            "la plaque garde son identité : {plate_height}"
        );
    }

    /// L'échelle de la plaque : elle cède d'abord, et elle ne cède jamais
    /// sous son identité.
    ///
    /// La décision de l'opérateur : la plaque reste visible — on sait
    /// toujours quel paquet est sélectionné — et c'est elle qui se réduit,
    /// avant que le catalogue ne soit réduit. Sa hauteur est donc bornée
    /// par ce que le catalogue garde, et son chrome se réduit partie par
    /// partie : le corps, puis les actions, puis les onglets.
    #[test]
    fn the_plate_yields_before_the_catalogue_is_reduced() {
        // The plate's ceiling: what is left once the catalogue has kept its
        // rows. The plate never takes more than that.
        for band in [320.0_f32, 356.0, 420.0, 680.0] {
            let (_, max) = nameplate_bounds(band);
            let catalogue = band
                - UiMetrics::SPLITTER_THICKNESS
                - nameplate_height(band, 1000.0);
            assert!(
                catalogue >= UiMetrics::CATALOGUE_MIN_HEIGHT - 0.5
                    || max <= UiMetrics::NAMEPLATE_MIN_HEIGHT + 0.5,
                "at {band} px, the catalogue keeps {catalogue} and the plate takes {max}"
            );
        }

        // Which rows the plate keeps is decided by the element that lays it
        // out, measured where the room is real
        // (`views::inspector::tests::the_plate_yields_part_by_part_in_a_real_window`).
    }
}
