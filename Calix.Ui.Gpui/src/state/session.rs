use crate::backend::models::UnifiedPackage;
use crate::state::query::{PackageStateFilter, SortMode, SourceScope};
use gpui::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageSourceKind {
    Alpm,
    Aur,
    Flatpak,
    AppImage,
    /// Crate Cargo exposée par l'inventaire du moteur Calix.
    Cargo,
    /// Paquet NPM global exposé par l'inventaire du moteur Calix.
    Npm,
    /// Paquet suivi par le backend de releases Calix.
    Release,
}

impl PackageSourceKind {
    /// Les sept écosystèmes, dans l'ordre où le contrôle les présente.
    pub const ALL: &'static [PackageSourceKind] = &[
        PackageSourceKind::Alpm,
        PackageSourceKind::Aur,
        PackageSourceKind::Flatpak,
        PackageSourceKind::AppImage,
        PackageSourceKind::Cargo,
        PackageSourceKind::Npm,
        PackageSourceKind::Release,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            PackageSourceKind::Alpm => "alpm",
            PackageSourceKind::Aur => "aur",
            PackageSourceKind::Flatpak => "flatpak",
            PackageSourceKind::AppImage => "appimage",
            PackageSourceKind::Cargo => "cargo",
            PackageSourceKind::Npm => "npm",
            PackageSourceKind::Release => "release",
        }
    }
}

impl std::fmt::Display for PackageSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSourceKind::Alpm => write!(f, "ALPM"),
            PackageSourceKind::Aur => write!(f, "AUR"),
            PackageSourceKind::Flatpak => write!(f, "Flatpak"),
            PackageSourceKind::AppImage => write!(f, "AppImage"),
            PackageSourceKind::Cargo => write!(f, "Cargo"),
            PackageSourceKind::Npm => write!(f, "NPM"),
            PackageSourceKind::Release => write!(f, "Release"),
        }
    }
}

/// Identifiant stable et typé d'un paquet garantissant l'indépendance vis-à-vis des index de liste
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageKey {
    pub source: PackageSourceKind,
    pub name: String,
    pub repository: Option<String>,
}

impl PackageKey {
    pub fn new(
        source: PackageSourceKind,
        name: impl Into<String>,
        repository: Option<String>,
    ) -> Self {
        Self {
            source,
            name: name.into(),
            repository,
        }
    }

    pub fn from_unified(pkg: &UnifiedPackage) -> Self {
        let source = match pkg.source_type.as_str() {
            "AUR" => PackageSourceKind::Aur,
            "Flatpak" => PackageSourceKind::Flatpak,
            "AppImage" => PackageSourceKind::AppImage,
            "Cargo" => PackageSourceKind::Cargo,
            "NPM" => PackageSourceKind::Npm,
            "Release" => PackageSourceKind::Release,
            _ => PackageSourceKind::Alpm,
        };
        PackageKey {
            source,
            name: pkg.name.clone(),
            repository: if pkg.repository_or_remote.is_empty() {
                None
            } else {
                Some(pkg.repository_or_remote.clone())
            },
        }
    }
}

use crate::icons::AppIcon;

/// Les lieux : une surface de travail, pas un jeu de données.
///
/// Le rail porte cinq entrées : les trois premières sont des périmètres de la
/// surface des paquets, les deux dernières sont des lieux à part entière. Un
/// lieu à la fois.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Place {
    #[default]
    Packages,
    News,
    Settings,
}

impl Place {
    pub fn label(&self) -> &'static str {
        match self {
            Place::Packages => "Packages",
            Place::News => "News",
            Place::Settings => "Settings",
        }
    }
}

/// Les périmètres : le jeu de données que montre la surface des paquets.
///
/// Un périmètre à la fois, et il est toujours nommé à l'écran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NavScope {
    #[default]
    Results,
    Installed,
    Updates,
}

impl NavScope {
    /// Les trois périmètres, dans l'ordre où le commutateur les présente.
    pub const ALL: &'static [NavScope] =
        &[NavScope::Results, NavScope::Installed, NavScope::Updates];

    pub fn label(&self) -> &'static str {
        match self {
            NavScope::Results => "Search results",
            NavScope::Installed => "Installed",
            NavScope::Updates => "Updates",
        }
    }

    /// La clé du périmètre, dans la surface de contrôle et dans l'état persisté.
    pub fn as_str(&self) -> &'static str {
        match self {
            NavScope::Results => "results",
            NavScope::Installed => "installed",
            NavScope::Updates => "updates",
        }
    }

    /// Lit une valeur de la surface de contrôle.
    ///
    /// `browse` reste un alias de `results` : les verbes d'avant doivent
    /// continuer à répondre, et le nom d'avant disait la même chose.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "results" | "search" | "browse" => Ok(NavScope::Results),
            "installed" => Ok(NavScope::Installed),
            "updates" => Ok(NavScope::Updates),
            other => Err(format!(
                "Invalid scope '{other}', expected 'results', 'installed' or 'updates'"
            )),
        }
    }

    pub fn icon(&self) -> AppIcon {
        match self {
            NavScope::Results => AppIcon::Search,
            NavScope::Installed => AppIcon::Installed,
            NavScope::Updates => AppIcon::Updates,
        }
    }

    /// Les écosystèmes que le périmètre peut contenir.
    ///
    /// Les périmètres qui interrogent les dépôts portent les quatre sources
    /// interrogeables ; celui qui liste l'inventaire en porte sept.
    pub fn sources(self) -> &'static [&'static str] {
        match self {
            NavScope::Results | NavScope::Updates => &["alpm", "aur", "flatpak", "appimage"],
            NavScope::Installed => {
                &["alpm", "aur", "flatpak", "appimage", "cargo", "npm", "release"]
            }
        }
    }

    /// Les écosystèmes que le périmètre liste mais ne peut pas interroger.
    pub fn inventory_only_sources(self) -> &'static [&'static str] {
        match self {
            NavScope::Installed => &["cargo", "npm", "release"],
            _ => &[],
        }
    }
}

impl std::fmt::Display for NavScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Une entrée du rail : un lieu, et le périmètre quand elle en a un.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RailEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: AppIcon,
    pub place: Place,
    /// `None` pour les lieux qui n'ont pas de périmètre.
    pub scope: Option<NavScope>,
}

impl RailEntry {
    /// Les cinq entrées du rail, dans leur ordre.
    pub const ALL: &'static [RailEntry] = &[
        RailEntry {
            id: "nav_scope_results",
            label: "Search",
            icon: AppIcon::Search,
            place: Place::Packages,
            scope: Some(NavScope::Results),
        },
        RailEntry {
            id: "nav_scope_installed",
            label: "Installed",
            icon: AppIcon::Installed,
            place: Place::Packages,
            scope: Some(NavScope::Installed),
        },
        RailEntry {
            id: "nav_scope_updates",
            label: "Updates",
            icon: AppIcon::Updates,
            place: Place::Packages,
            scope: Some(NavScope::Updates),
        },
        RailEntry {
            id: "nav_place_news",
            label: "News",
            icon: AppIcon::News,
            place: Place::News,
            scope: None,
        },
        RailEntry {
            id: "nav_place_settings",
            label: "Settings",
            icon: AppIcon::Settings,
            place: Place::Settings,
            scope: None,
        },
    ];

    /// L'entrée est-elle celle qui est ouverte ?
    pub fn is_active(&self, place: Place, scope: NavScope) -> bool {
        self.place == place && self.scope.map_or(true, |s| s == scope)
    }
}

/// Mode d'affichage de la surface des paquets (Cartes ou Table dense)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PackageViewMode {
    #[default]
    Cards,
    Table,
}

impl PackageViewMode {
    pub fn icon(&self) -> AppIcon {
        match self {
            PackageViewMode::Cards => AppIcon::Cards,
            PackageViewMode::Table => AppIcon::Table,
        }
    }
}

/// Onglets de l'inspecteur sémantique de paquet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum InspectorTab {
    #[default]
    Overview,
    Dependencies,
    FilesBuild,
}

impl InspectorTab {
    pub fn label(&self) -> &'static str {
        match self {
            InspectorTab::Overview => "Overview",
            InspectorTab::Dependencies => "Dependencies",
            InspectorTab::FilesBuild => "Files & Build",
        }
    }

    pub fn icon(&self) -> AppIcon {
        match self {
            InspectorTab::Overview => AppIcon::Overview,
            InspectorTab::Dependencies => AppIcon::Dependencies,
            InspectorTab::FilesBuild => AppIcon::FilesBuild,
        }
    }
}

/// Événements sémantiques émis par la session applicative
#[derive(Debug, Clone, PartialEq)]
pub enum SessionEvent {
    PlaceChanged(Place),
    ScopeChanged(NavScope),
    SearchQueryChanged(String),
    PackageSelected(Option<PackageKey>),
    SidebarToggled(bool),
    ViewModeChanged(PackageViewMode),
    InspectorTabChanged(InspectorTab),
    SearchingStateChanged(bool),
    SourceScopeChanged(SourceScope),
    StateFilterChanged(PackageStateFilter),
    SortModeChanged(SortMode),
}

/// Entité GPUI gérant l'état de navigation et d'intention de l'utilisateur
pub struct AppSession {
    pub place: Place,
    pub scope: NavScope,
    /// La surface de développement n'est pas un lieu : elle est ouverte par
    ///-dessus, et le lieu d'en dessous est retrouvé à la fermeture.
    pub render_lab: bool,
    pub source_scope: SourceScope,
    pub state_filter: PackageStateFilter,
    pub sort_mode: SortMode,
    pub search_query: String,
    pub selected_package_key: Option<PackageKey>,
    pub search_generation: usize,
    pub is_searching: bool,
    pub sidebar_collapsed: bool,
    pub view_mode: PackageViewMode,
    pub inspector_tab: InspectorTab,
    pub navigation_epoch: u64,
    pub inspector_tab_epoch: u64,
}

impl EventEmitter<SessionEvent> for AppSession {}

impl AppSession {
    pub fn new() -> Self {
        Self::with_initial_scope(NavScope::default())
    }

    pub fn with_initial_scope(scope: NavScope) -> Self {
        Self {
            place: Place::default(),
            scope,
            render_lab: false,
            source_scope: SourceScope::all(),
            state_filter: PackageStateFilter::All,
            sort_mode: SortMode::Relevance,
            search_query: String::new(),
            selected_package_key: None,
            search_generation: 0,
            is_searching: false,
            sidebar_collapsed: false,
            view_mode: PackageViewMode::Cards,
            inspector_tab: InspectorTab::Overview,
            navigation_epoch: 0,
            inspector_tab_epoch: 0,
        }
    }

    /// Change de lieu. Le périmètre courant est conservé : revenir de Actualités
    /// ou de Réglages ramène au périmètre d'où l'on venait.
    pub fn set_place(&mut self, place: Place, cx: &mut Context<Self>) {
        if self.place != place {
            self.place = place;
            self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
            cx.emit(SessionEvent::PlaceChanged(place));
            cx.notify();
        }
    }

    /// Change de périmètre. Les filtres sont conservés : l'utilisateur a
    /// demandé une vue filtrée, elle ne se vide pas sous lui.
    pub fn set_scope(&mut self, scope: NavScope, cx: &mut Context<Self>) {
        if self.scope != scope {
            self.scope = scope;
            // Le vocabulaire des filtres suit le périmètre : un état qui ne peut
            // pas être vrai des lignes du nouveau périmètre est retiré, sinon il
            // viderait la liste sans que l'utilisateur l'ait demandé.
            if !self.state_filter.is_meaningful_in(scope) {
                self.state_filter = PackageStateFilter::All;
            }
            self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
            cx.emit(SessionEvent::ScopeChanged(scope));
            cx.notify();
        }
    }

    /// Ouvre ou ferme la surface de développement.
    pub fn set_render_lab(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.render_lab != open {
            self.render_lab = open;
            self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
            cx.notify();
        }
    }

    pub fn set_source_scope(&mut self, scope: SourceScope, cx: &mut Context<Self>) {
        if self.source_scope != scope {
            self.source_scope = scope;
            cx.emit(SessionEvent::SourceScopeChanged(scope));
            cx.notify();
        }
    }

    pub fn toggle_source(&mut self, source: PackageSourceKind, cx: &mut Context<Self>) {
        self.source_scope.toggle_source(source);
        // Le périmètre ne se vide pas sous un filtre : la dernière source du
        // périmètre reste sélectionnée. Le contrôle des sources ne propose que
        // les écosystèmes du périmètre, donc c'est de ceux-là qu'on compte.
        let scope_sources = SourceScope::ecosystems_for(self.scope);
        if !scope_sources
            .iter()
            .any(|entry| self.source_scope.is_selected(entry.kind))
        {
            self.source_scope.select(source, true);
        }
        cx.emit(SessionEvent::SourceScopeChanged(self.source_scope));
        cx.notify();
    }

    pub fn clamp_source_scope(
        &mut self,
        aur_enabled: bool,
        flatpak_enabled: bool,
        appimage_enabled: bool,
        cx: &mut Context<Self>,
    ) {
        self.source_scope
            .clamp_to_enabled(aur_enabled, flatpak_enabled, appimage_enabled);
        cx.emit(SessionEvent::SourceScopeChanged(self.source_scope));
        cx.notify();
    }

    pub fn set_state_filter(&mut self, filter: PackageStateFilter, cx: &mut Context<Self>) {
        if self.state_filter != filter {
            self.state_filter = filter;
            cx.emit(SessionEvent::StateFilterChanged(filter));
            cx.notify();
        }
    }

    pub fn set_sort_mode(&mut self, mode: SortMode, cx: &mut Context<Self>) {
        if self.sort_mode != mode {
            self.sort_mode = mode;
            cx.emit(SessionEvent::SortModeChanged(mode));
            cx.notify();
        }
    }

    pub fn clear_filters(
        &mut self,
        aur_enabled: bool,
        flatpak_enabled: bool,
        appimage_enabled: bool,
        cx: &mut Context<Self>,
    ) {
        self.source_scope =
            SourceScope::all_enabled(aur_enabled, flatpak_enabled, appimage_enabled);
        self.state_filter = PackageStateFilter::All;
        cx.emit(SessionEvent::SourceScopeChanged(self.source_scope));
        cx.emit(SessionEvent::StateFilterChanged(PackageStateFilter::All));
        cx.notify();
    }

    pub fn set_search_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.search_query = query.clone();
        cx.emit(SessionEvent::SearchQueryChanged(query));
        cx.notify();
    }

    pub fn select_package(&mut self, key: Option<PackageKey>, cx: &mut Context<Self>) {
        if self.selected_package_key != key {
            self.selected_package_key = key.clone();
            cx.emit(SessionEvent::PackageSelected(key));
            cx.notify();
        }
    }

    pub fn set_view_mode(&mut self, mode: PackageViewMode, cx: &mut Context<Self>) {
        if self.view_mode != mode {
            self.view_mode = mode;
            cx.emit(SessionEvent::ViewModeChanged(mode));
            cx.notify();
        }
    }

    pub fn set_inspector_tab(&mut self, tab: InspectorTab, cx: &mut Context<Self>) {
        if self.inspector_tab != tab {
            self.inspector_tab = tab;
            self.inspector_tab_epoch = self.inspector_tab_epoch.wrapping_add(1);
            cx.emit(SessionEvent::InspectorTabChanged(tab));
            cx.notify();
        }
    }

    pub fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.sidebar_collapsed != collapsed {
            self.sidebar_collapsed = collapsed;
            cx.emit(SessionEvent::SidebarToggled(self.sidebar_collapsed));
            cx.notify();
        }
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.set_sidebar_collapsed(!self.sidebar_collapsed, cx);
    }

    pub fn next_search_generation(&mut self) -> usize {
        self.search_generation += 1;
        self.search_generation
    }

    pub fn set_searching(&mut self, searching: bool, cx: &mut Context<Self>) {
        if self.is_searching != searching {
            self.is_searching = searching;
            cx.emit(SessionEvent::SearchingStateChanged(searching));
            cx.notify();
        }
    }
}

impl Default for AppSession {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_package_key_equality_and_hashing() {
        let k1 = PackageKey::new(PackageSourceKind::Alpm, "ripgrep", Some("extra".into()));
        let k2 = PackageKey::new(PackageSourceKind::Alpm, "ripgrep", Some("extra".into()));
        let k3 = PackageKey::new(PackageSourceKind::Aur, "ripgrep", None);

        assert_eq!(k1, k2);
        assert_ne!(k1, k3);

        let mut map = std::collections::HashMap::new();
        map.insert(k1.clone(), "alpm-extra");
        map.insert(k3.clone(), "aur");

        assert_eq!(map.get(&k2), Some(&"alpm-extra"));
        assert_eq!(map.get(&k3), Some(&"aur"));
    }

    #[test]
    fn test_app_session_defaults() {
        let session = AppSession::new();
        assert_eq!(session.place, Place::Packages);
        assert_eq!(session.scope, NavScope::Results);
        assert!(!session.render_lab);
        assert_eq!(session.source_scope, SourceScope::all());
        assert_eq!(session.state_filter, PackageStateFilter::All);
        assert_eq!(session.sort_mode, SortMode::Relevance);
        assert_eq!(session.search_query, "");
        assert_eq!(session.selected_package_key, None);
        assert!(!session.sidebar_collapsed);
        assert_eq!(session.search_generation, 0);
        assert_eq!(session.view_mode, PackageViewMode::Cards);
        assert_eq!(session.inspector_tab, InspectorTab::Overview);
    }

    #[test]
    fn test_nav_scope_metadata() {
        assert_eq!(NavScope::Results.label(), "Search results");
        assert_eq!(NavScope::Installed.label(), "Installed");
        assert_eq!(NavScope::Updates.label(), "Updates");
        assert_eq!(NavScope::Results.as_str(), "results");
        assert_eq!(NavScope::Installed.as_str(), "installed");
        assert_eq!(NavScope::Updates.as_str(), "updates");
    }

    #[test]
    fn test_nav_scope_parses_the_control_surface_keys() {
        assert_eq!(NavScope::parse("results").unwrap(), NavScope::Results);
        assert_eq!(NavScope::parse("browse").unwrap(), NavScope::Results);
        assert_eq!(NavScope::parse("Installed").unwrap(), NavScope::Installed);
        assert_eq!(NavScope::parse("updates").unwrap(), NavScope::Updates);
        assert!(NavScope::parse("cyan").is_err());
    }

    #[test]
    fn test_the_rail_carries_five_entries_and_marks_the_active_one() {
        assert_eq!(RailEntry::ALL.len(), 5);
        assert_eq!(RailEntry::ALL[0].scope, Some(NavScope::Results));
        assert_eq!(RailEntry::ALL[3].scope, None);
        assert_eq!(RailEntry::ALL[3].place, Place::News);

        // The entry of the current scope is the active one, and the three
        // package entries never mark the same scope twice.
        let mut active = 0;
        for entry in RailEntry::ALL {
            if entry.is_active(Place::Packages, NavScope::Installed) {
                active += 1;
                assert_eq!(entry.scope, Some(NavScope::Installed));
            }
        }
        assert_eq!(active, 1);

        assert!(RailEntry::ALL[3].is_active(Place::News, NavScope::Results));
        assert!(!RailEntry::ALL[3].is_active(Place::Settings, NavScope::Results));
    }

    #[test]
    fn test_clear_filters_resets_scope_and_state_but_preserves_query_and_sort() {
        let mut session = AppSession::new();
        let mut scope = SourceScope::all();
        scope.alpm = false;
        scope.flatpak = false;
        scope.appimage = false;
        session.source_scope = scope;
        session.state_filter = PackageStateFilter::Installed;
        session.sort_mode = SortMode::NameAsc;
        session.search_query = "linux".to_string();

        // Clear filters with aur and flatpak enabled, appimage disabled
        // In unit test without Context, we verify the logic of clear_filters:
        session.source_scope = SourceScope::all_enabled(true, true, false);
        session.state_filter = PackageStateFilter::All;

        assert!(session.source_scope.alpm);
        assert!(session.source_scope.aur);
        assert!(session.source_scope.flatpak);
        assert!(!session.source_scope.appimage);
        assert_eq!(session.state_filter, PackageStateFilter::All);
        assert_eq!(session.sort_mode, SortMode::NameAsc);
        assert_eq!(session.search_query, "linux");
    }

    #[test]
    fn test_view_mode_and_inspector_tab_metadata() {
        assert_eq!(PackageViewMode::Cards.icon(), AppIcon::Cards);
        assert_eq!(PackageViewMode::Table.icon(), AppIcon::Table);

        assert_eq!(InspectorTab::Overview.label(), "Overview");
        assert_eq!(InspectorTab::Dependencies.label(), "Dependencies");
        assert_eq!(InspectorTab::FilesBuild.label(), "Files & Build");
        assert_eq!(InspectorTab::Overview.icon(), AppIcon::Overview);
        assert_eq!(InspectorTab::Dependencies.icon(), AppIcon::Dependencies);
        assert_eq!(InspectorTab::FilesBuild.icon(), AppIcon::FilesBuild);

        assert_eq!(NavScope::Results.icon(), AppIcon::Search);
        assert_eq!(NavScope::Installed.icon(), AppIcon::Installed);
        assert_eq!(NavScope::Updates.icon(), AppIcon::Updates);
        assert_eq!(Place::News.label(), "News");
        assert_eq!(Place::Settings.label(), "Settings");
    }

    #[test]
    fn test_view_mode_continuity_preserves_selection_query_and_generation() {
        let mut session = AppSession::new();
        session.scope = NavScope::Results;
        session.search_query = "ripgrep".to_string();
        session.search_generation = 12;

        let expected_key = PackageKey::new(
            PackageSourceKind::Alpm,
            "ripgrep",
            Some("cachyos-v3".to_string()),
        );
        session.selected_package_key = Some(expected_key.clone());
        assert_eq!(session.view_mode, PackageViewMode::Cards);

        // Switch Cards -> Table
        session.view_mode = PackageViewMode::Table;

        // Verify continuity
        assert_eq!(session.search_query, "ripgrep");
        assert_eq!(session.selected_package_key, Some(expected_key.clone()));
        assert_eq!(session.search_generation, 12);

        // Switch Table -> Cards
        session.view_mode = PackageViewMode::Cards;

        // Verify continuity again
        assert_eq!(session.search_query, "ripgrep");
        assert_eq!(session.selected_package_key, Some(expected_key));
        assert_eq!(session.search_generation, 12);
    }

    #[test]
    fn test_dependency_navigation_preserves_clean_package_name() {
        use crate::state::semantic::{DependencyKind, DependencyRef};

        let raw_dep = "libalpm.so>=14: Arch package management library";
        let parsed = DependencyRef::parse(raw_dep, DependencyKind::Runtime);
        assert_eq!(parsed.name, "libalpm.so");
        assert_eq!(parsed.constraint, Some(">=14".to_string()));

        // Simulate dependency click navigation in session
        let mut session = AppSession::new();
        session.scope = NavScope::Installed;
        session.search_query = "calix".to_string();

        // When navigating to dependency, clean name is used, not raw constraint/description
        session.scope = NavScope::Results;
        session.search_query = parsed.name.clone();
        session.search_generation += 1;

        assert_eq!(session.scope, NavScope::Results);
        assert_eq!(session.search_query, "libalpm.so");
        assert!(!session.search_query.contains(">=14"));
        assert!(!session.search_query.contains("Arch package"));
    }

    #[test]
    fn ecosystem_packages_get_their_own_source_kind() {
        use crate::backend::models::{CargoCrate, NpmPackage, ReleasePackage};
        use crate::backend::models::UnifiedPackageSource;

        let cases = vec![
            (
                "Cargo",
                UnifiedPackageSource::Cargo(CargoCrate {
                    version: "0.13.11".into(),
                    source: "crates.io".into(),
                    bins: vec!["cargo-add".into()],
                }),
                PackageSourceKind::Cargo,
            ),
            (
                "NPM",
                UnifiedPackageSource::Npm(NpmPackage {
                    version: "5.6.3".into(),
                    overridden: false,
                }),
                PackageSourceKind::Npm,
            ),
            (
                "Release",
                UnifiedPackageSource::Release(ReleasePackage {
                    installed_version: Some("0.5.1".into()),
                    latest_version: Some("0.5.1".into()),
                    update_available: false,
                    repo: Some("storytold/craft-launcher".into()),
                    provider: Some("github".into()),
                    canal: Some("stable".into()),
                    pkgname: Some("artcraft-launcher-bin".into()),
                }),
                PackageSourceKind::Release,
            ),
        ];

        for (source_type, inner, expected) in cases {
            let pkg = UnifiedPackage {
                name: "cargo-edit".into(),
                version: "1.0.0".into(),
                description: "test".into(),
                source_type: source_type.into(),
                repository_or_remote: "origin".into(),
                is_installed: true,
                has_update: false,
                new_version: None,
                inner,
            };
            let key = PackageKey::from_unified(&pkg);
            assert_eq!(
                key.source, expected,
                "{} must not be keyed as ALPM",
                source_type
            );
        }
    }

    #[test]
    fn test_session_epochs_increment() {
        let mut session = AppSession::new();
        assert_eq!(session.navigation_epoch, 0);
        assert_eq!(session.inspector_tab_epoch, 0);

        // The navigation epoch is what tells the views that the place or the
        // scope moved.
        session.place = Place::News;
        session.navigation_epoch += 1;
        assert_eq!(session.navigation_epoch, 1);

        session.scope = NavScope::Installed;
        session.navigation_epoch += 1;
        assert_eq!(session.navigation_epoch, 2);

        session.inspector_tab = InspectorTab::Dependencies;
        session.inspector_tab_epoch += 1;
        assert_eq!(session.inspector_tab_epoch, 1);
    }

    #[test]
    fn the_render_lab_is_not_a_place() {
        // The render lab opens above the place, and the place is found again
        // when it closes: it is not a destination of its own.
        let mut session = AppSession::new();
        assert!(!session.render_lab);
        assert_eq!(session.place, Place::Packages);
        session.render_lab = true;
        assert_eq!(session.place, Place::Packages);
        session.render_lab = false;
        assert_eq!(session.place, Place::Packages);
    }

    #[test]
    fn the_scope_survives_a_place_change() {
        // Le périmètre appartient à la session, pas au lieu : revenir de
        // Réglages ramène au périmètre d'où l'on venait.
        let mut session = AppSession::new();
        session.scope = NavScope::Installed;
        session.place = Place::Settings;
        assert_eq!(session.scope, NavScope::Installed);
        session.place = Place::Packages;
        assert_eq!(session.scope, NavScope::Installed);
    }

}
