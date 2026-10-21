use crate::backend::models::UnifiedPackage;
use crate::state::session::{NavScope, PackageSourceKind};
use serde::{Deserialize, Serialize};

/// Portée de filtrage des sources (sélection multiple ou unitaire de dépôts)
///
/// Les sept écosystèmes que Calix connaît ont leur drapeau. Les quatre
/// premiers s'interrogent dans les dépôts ; les trois autres ne se listent
/// qu'installés : ils sont dans l'inventaire, ils ne sont pas des sources de
/// recherche. Un filtre qui ne représente pas les lignes qu'il filtre est un
/// mensonge — d'où les trois drapeaux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceScope {
    pub alpm: bool,
    pub aur: bool,
    pub flatpak: bool,
    pub appimage: bool,
    /// Les crates Cargo de l'inventaire : listés, jamais interrogés.
    pub cargo: bool,
    /// Les paquets NPM globaux de l'inventaire : listés, jamais interrogés.
    pub npm: bool,
    /// Les releases suivis de l'inventaire : listés, jamais interrogés.
    pub release: bool,
}

impl Default for SourceScope {
    fn default() -> Self {
        Self::all()
    }
}

impl SourceScope {
    pub fn all() -> Self {
        Self {
            alpm: true,
            aur: true,
            flatpak: true,
            appimage: true,
            cargo: true,
            npm: true,
            release: true,
        }
    }

    pub fn all_enabled(aur_enabled: bool, flatpak_enabled: bool, appimage_enabled: bool) -> Self {
        let mut s = Self::all();
        s.clamp_to_enabled(aur_enabled, flatpak_enabled, appimage_enabled);
        s
    }

    pub fn is_all(&self) -> bool {
        self.alpm
            && self.aur
            && self.flatpak
            && self.appimage
            && self.cargo
            && self.npm
            && self.release
    }

    pub fn is_all_enabled(
        &self,
        aur_enabled: bool,
        flatpak_enabled: bool,
        appimage_enabled: bool,
    ) -> bool {
        self.alpm
            && (!aur_enabled || self.aur)
            && (!flatpak_enabled || self.flatpak)
            && (!appimage_enabled || self.appimage)
    }

    /// L'écosystème est-il sélectionné ?
    pub fn is_selected(&self, source: PackageSourceKind) -> bool {
        self.flag(source)
    }

    /// Sélectionne ou retire un écosystème.
    pub fn select(&mut self, source: PackageSourceKind, selected: bool) {
        self.set_flag(source, selected);
    }

    /// Le drapeau d'un écosystème.
    fn flag(&self, source: PackageSourceKind) -> bool {
        match source {
            PackageSourceKind::Alpm => self.alpm,
            PackageSourceKind::Aur => self.aur,
            PackageSourceKind::Flatpak => self.flatpak,
            PackageSourceKind::AppImage => self.appimage,
            PackageSourceKind::Cargo => self.cargo,
            PackageSourceKind::Npm => self.npm,
            PackageSourceKind::Release => self.release,
        }
    }

    /// Bascule un écosystème.
    ///
    /// Le dernier écosystème sélectionné ne s'éteint pas : un filtre qui
    /// ne montre plus rien n'est pas un filtre.
    pub fn toggle_source(&mut self, source: PackageSourceKind) {
        let other_selected = PackageSourceKind::ALL
            .iter()
            .any(|kind| *kind != source && self.flag(*kind));
        let selected = self.flag(source);
        if !selected || other_selected {
            self.set_flag(source, !selected);
        }
        // Invariant strict : jamais zéro source sélectionnée
        if !PackageSourceKind::ALL.iter().any(|kind| self.flag(*kind)) {
            self.set_flag(PackageSourceKind::Alpm, true);
        }
    }

    fn set_flag(&mut self, source: PackageSourceKind, value: bool) {
        match source {
            PackageSourceKind::Alpm => self.alpm = value,
            PackageSourceKind::Aur => self.aur = value,
            PackageSourceKind::Flatpak => self.flatpak = value,
            PackageSourceKind::AppImage => self.appimage = value,
            PackageSourceKind::Cargo => self.cargo = value,
            PackageSourceKind::Npm => self.npm = value,
            PackageSourceKind::Release => self.release = value,
        }
    }

    /// Une ligne est-elle d'une source sélectionnée ?
    ///
    /// Chaque écosystème a son drapeau : une ligne Cargo disparaît quand Cargo
    /// est décoché, et non quand ALPM l'est.
    pub fn contains_str(&self, source_type: &str) -> bool {
        match source_type {
            "Standard" | "Official" | "ALPM" => self.alpm,
            "AUR" => self.aur,
            "Flatpak" => self.flatpak,
            "AppImage" => self.appimage,
            "Cargo" => self.cargo,
            "NPM" => self.npm,
            "Release" => self.release,
            _ => self.alpm,
        }
    }

    pub fn clamp_to_enabled(
        &mut self,
        aur_enabled: bool,
        flatpak_enabled: bool,
        appimage_enabled: bool,
    ) {
        if !aur_enabled {
            self.aur = false;
        }
        if !flatpak_enabled {
            self.flatpak = false;
        }
        if !appimage_enabled {
            self.appimage = false;
        }
        if !PackageSourceKind::ALL.iter().any(|kind| self.flag(*kind)) {
            self.alpm = true;
        }
    }

    /// Les sources de recherche sélectionnées.
    ///
    /// Les quatre dépôts que Calix interroge : les trois écosystèmes d'inventaire
    /// n'ont pas de dépôt à interroger.
    pub fn enabled_sources(
        &self,
        aur_enabled: bool,
        flatpak_enabled: bool,
        appimage_enabled: bool,
    ) -> Vec<PackageSourceKind> {
        let mut list = Vec::new();
        if self.alpm {
            list.push(PackageSourceKind::Alpm);
        }
        if self.aur && aur_enabled {
            list.push(PackageSourceKind::Aur);
        }
        if self.flatpak && flatpak_enabled {
            list.push(PackageSourceKind::Flatpak);
        }
        if self.appimage && appimage_enabled {
            list.push(PackageSourceKind::AppImage);
        }
        list
    }

    /// Les sources sélectionnées, parmi celles que le périmètre peut contenir.
    ///
    /// Le périmètre décide de ce qu'il contient ; le filtre décide de ce qui
    /// reste. Un périmètre qui ne contient pas de Cargo n'a pas de ligne Cargo
    /// à cacher.
    pub fn sources_for(
        &self,
        scope: NavScope,
        aur_enabled: bool,
        flatpak_enabled: bool,
        appimage_enabled: bool,
    ) -> Vec<PackageSourceKind> {
        let mut list = Vec::new();
        for kind in scope.sources() {
            let kind = match *kind {
                "alpm" => PackageSourceKind::Alpm,
                "aur" => PackageSourceKind::Aur,
                "flatpak" => PackageSourceKind::Flatpak,
                "appimage" => PackageSourceKind::AppImage,
                "cargo" => PackageSourceKind::Cargo,
                "npm" => PackageSourceKind::Npm,
                _ => PackageSourceKind::Release,
            };
            if !self.flag(kind) {
                continue;
            }
            match kind {
                PackageSourceKind::Alpm => list.push(kind),
                PackageSourceKind::Aur if aur_enabled => list.push(kind),
                PackageSourceKind::Flatpak if flatpak_enabled => list.push(kind),
                PackageSourceKind::AppImage if appimage_enabled => list.push(kind),
                _ => list.push(kind),
            }
        }
        list
    }
}

/// Filtre d'état local des paquets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum PackageStateFilter {
    #[default]
    All,
    Installed,
    NotInstalled,
    UpdatesAvailable,
}

impl PackageStateFilter {
    pub fn label(&self) -> &'static str {
        match self {
            PackageStateFilter::All => "All States",
            PackageStateFilter::Installed => "Installed",
            PackageStateFilter::NotInstalled => "Not Installed",
            PackageStateFilter::UpdatesAvailable => "Updates Available",
        }
    }

    pub fn matches(&self, is_installed: bool, has_update: bool) -> bool {
        match self {
            PackageStateFilter::All => true,
            PackageStateFilter::Installed => is_installed,
            PackageStateFilter::NotInstalled => !is_installed,
            PackageStateFilter::UpdatesAvailable => has_update,
        }
    }

    /// Les états qui peuvent être vrais des lignes du périmètre.
    ///
    /// Un état impossible dans le périmètre n'est pas proposé : dans Installés,
    /// aucune ligne n'est « non installée » ; dans Mises à jour, chaque ligne a
    /// une mise à jour.
    pub fn options_for(scope: NavScope) -> &'static [PackageStateFilter] {
        match scope {
            NavScope::Results => &[
                PackageStateFilter::All,
                PackageStateFilter::Installed,
                PackageStateFilter::NotInstalled,
                PackageStateFilter::UpdatesAvailable,
            ],
            NavScope::Installed => &[PackageStateFilter::All, PackageStateFilter::UpdatesAvailable],
            NavScope::Updates => &[PackageStateFilter::All],
        }
    }

    /// L'état a-t-il un sens dans ce périmètre ?
    pub fn is_meaningful_in(self, scope: NavScope) -> bool {
        Self::options_for(scope).contains(&self)
    }
}

/// Mode de tri déterministe côté client (zéro ré-interrogation backend)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SortMode {
    #[default]
    Relevance,
    NameAsc,
    NameDesc,
    Source,
    InstalledFirst,
    UpdatesFirst,
}

impl SortMode {
    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Relevance => "Relevance",
            SortMode::NameAsc => "Name (A–Z)",
            SortMode::NameDesc => "Name (Z–A)",
            SortMode::Source => "Source Backend",
            SortMode::InstalledFirst => "Installed First",
            SortMode::UpdatesFirst => "Updates First",
        }
    }

    /// Trie une tranche mutable de paquets en place avec départage déterministe strict
    pub fn sort_packages(&self, packages: &mut [UnifiedPackage]) {
        match self {
            SortMode::Relevance => {
                // Conserve l'ordonnancement originel du score backend
            }
            SortMode::NameAsc => {
                packages.sort_by(|a, b| {
                    a.name
                        .to_lowercase()
                        .cmp(&b.name.to_lowercase())
                        .then_with(|| a.name.cmp(&b.name))
                        .then_with(|| a.source_type.cmp(&b.source_type))
                });
            }
            SortMode::NameDesc => {
                packages.sort_by(|a, b| {
                    b.name
                        .to_lowercase()
                        .cmp(&a.name.to_lowercase())
                        .then_with(|| b.name.cmp(&a.name))
                        .then_with(|| a.source_type.cmp(&b.source_type))
                });
            }
            SortMode::Source => {
                packages.sort_by(|a, b| {
                    let source_rank = |src: &str| match src {
                        "Standard" | "Official" | "ALPM" => 0,
                        "AUR" => 1,
                        "Flatpak" => 2,
                        "AppImage" => 3,
                        _ => 4,
                    };
                    source_rank(&a.source_type)
                        .cmp(&source_rank(&b.source_type))
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                        .then_with(|| a.name.cmp(&b.name))
                });
            }
            SortMode::InstalledFirst => {
                packages.sort_by(|a, b| {
                    let a_inst = a.is_installed;
                    let b_inst = b.is_installed;
                    b_inst
                        .cmp(&a_inst) // true avant false
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                        .then_with(|| a.name.cmp(&b.name))
                        .then_with(|| a.source_type.cmp(&b.source_type))
                });
            }
            SortMode::UpdatesFirst => {
                packages.sort_by(|a, b| {
                    let a_up = a.has_update;
                    let b_up = b.has_update;
                    b_up.cmp(&a_up)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                        .then_with(|| a.name.cmp(&b.name))
                        .then_with(|| a.source_type.cmp(&b.source_type))
                });
            }
        }
    }
}

/// Point de rupture adaptatif de la géométrie de l'établi de recherche
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkbenchBreakpoint {
    Wide,   // >= 640px : puces rapides + popover filtres + tri + vue
    Medium, // 460px .. 640px : popover filtres + état + tri + vue
    Narrow, // < 460px : popover filtres compact + tri compact + vue
}

impl WorkbenchBreakpoint {
    pub fn from_width(width: f32) -> Self {
        if width >= 640.0 {
            WorkbenchBreakpoint::Wide
        } else if width >= 460.0 {
            WorkbenchBreakpoint::Medium
        } else {
            WorkbenchBreakpoint::Narrow
        }
    }
}

/// Les lignes d'un périmètre, filtrées et triées comme l'interface les filtre.
///
/// Le poste de travail et l'en-tête de contexte appellent la même fonction :
/// le compte que l'en-tête annonce est celui que la liste montre.
/// La ligne correspond-elle à la requête ?
///
/// La requête réduit le périmètre courant : dans Installés et Mises à jour, les
/// lignes sont déjà là, la requête les réduit.
pub fn matches_query(package: &UnifiedPackage, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    package.name.to_lowercase().contains(&query)
        || package.description.to_lowercase().contains(&query)
}

pub fn visible_packages(
    packages: &[UnifiedPackage],
    source_scope: &SourceScope,
    state_filter: PackageStateFilter,
    sort_mode: SortMode,
) -> Vec<UnifiedPackage> {
    if source_scope.is_all()
        && state_filter == PackageStateFilter::All
        && sort_mode == SortMode::Relevance
    {
        return packages.to_vec();
    }
    let mut filtered: Vec<UnifiedPackage> = packages
        .iter()
        .filter(|p| {
            source_scope.contains_str(&p.source_type)
                && state_filter.matches(p.is_installed, p.has_update)
        })
        .cloned()
        .collect();
    sort_mode.sort_packages(&mut filtered);
    filtered
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    fn make_pkg(
        name: &str,
        source: &str,
        is_installed: bool,
        has_update: bool,
        version: &str,
    ) -> UnifiedPackage {
        UnifiedPackage {
            name: name.to_string(),
            version: version.to_string(),
            description: "test".to_string(),
            source_type: source.to_string(),
            repository_or_remote: "".to_string(),
            is_installed,
            has_update,
            new_version: None,
            inner: crate::backend::models::UnifiedPackageSource::Standard(
                crate::backend::models::AlpmPackage::default(),
            ),
        }
    }

    #[test]
    fn test_sort_mode_relevance_name_source_installed() {
        let mut pkgs = vec![
            make_pkg("zeta", "AUR", false, false, "1.0"),
            make_pkg("alpha", "Flatpak", true, false, "1.0"),
            make_pkg("beta", "Standard", true, true, "2.0"), // has update!
        ];

        // NameAsc
        SortMode::NameAsc.sort_packages(&mut pkgs);
        assert_eq!(pkgs[0].name, "alpha");
        assert_eq!(pkgs[1].name, "beta");
        assert_eq!(pkgs[2].name, "zeta");

        // NameDesc
        SortMode::NameDesc.sort_packages(&mut pkgs);
        assert_eq!(pkgs[0].name, "zeta");
        assert_eq!(pkgs[1].name, "beta");
        assert_eq!(pkgs[2].name, "alpha");

        // Source: Standard -> AUR -> Flatpak
        SortMode::Source.sort_packages(&mut pkgs);
        assert_eq!(pkgs[0].source_type, "Standard");
        assert_eq!(pkgs[1].source_type, "AUR");
        assert_eq!(pkgs[2].source_type, "Flatpak");

        // InstalledFirst: alpha & beta before zeta
        SortMode::InstalledFirst.sort_packages(&mut pkgs);
        assert!(pkgs[0].is_installed);
        assert!(pkgs[1].is_installed);
        assert!(!pkgs[2].is_installed);

        // UpdatesFirst: beta (has_update) before others
        SortMode::UpdatesFirst.sort_packages(&mut pkgs);
        assert_eq!(pkgs[0].name, "beta");
        assert!(pkgs[0].has_update);
    }

    #[test]
    fn test_sort_mode_stable_tie_breaking() {
        let mut pkgs = vec![
            make_pkg("pkg", "Flatpak", false, false, "1.0"),
            make_pkg("pkg", "AUR", false, false, "1.0"),
            make_pkg("pkg", "Standard", false, false, "1.0"),
        ];

        SortMode::NameAsc.sort_packages(&mut pkgs);
        // Name is identical, secondary tie-breaker is source_type alphabetical ("AUR", "Flatpak", "Standard")
        assert_eq!(pkgs[0].source_type, "AUR");
        assert_eq!(pkgs[1].source_type, "Flatpak");
        assert_eq!(pkgs[2].source_type, "Standard");
    }

    #[test]
    fn test_package_state_filter_installed_updates() {
        let installed_pkg = make_pkg("inst", "Standard", true, false, "1.0");
        let update_pkg = make_pkg("upd", "Standard", true, true, "2.0");
        let not_inst_pkg = make_pkg("not_inst", "Standard", false, false, "1.0");

        let filter_all = PackageStateFilter::All;
        assert!(filter_all.matches(installed_pkg.is_installed, installed_pkg.has_update));
        assert!(filter_all.matches(not_inst_pkg.is_installed, not_inst_pkg.has_update));

        let filter_inst = PackageStateFilter::Installed;
        assert!(filter_inst.matches(installed_pkg.is_installed, installed_pkg.has_update));
        assert!(!filter_inst.matches(not_inst_pkg.is_installed, not_inst_pkg.has_update));

        let filter_not_inst = PackageStateFilter::NotInstalled;
        assert!(!filter_not_inst.matches(installed_pkg.is_installed, installed_pkg.has_update));
        assert!(filter_not_inst.matches(not_inst_pkg.is_installed, not_inst_pkg.has_update));

        let filter_upd = PackageStateFilter::UpdatesAvailable;
        assert!(filter_upd.matches(update_pkg.is_installed, update_pkg.has_update));
        assert!(!filter_upd.matches(installed_pkg.is_installed, installed_pkg.has_update));
    }

    #[test]
    fn test_multi_source_scope_selection_and_disabled_exclusion() {
        let mut scope = SourceScope::all();
        assert!(scope.is_all());
        assert_eq!(scope.enabled_sources(true, true, true).len(), 4);
        assert!(scope.contains_str("Standard"));
        assert!(scope.contains_str("AUR"));
        assert!(scope.contains_str("Flatpak"));
        assert!(scope.contains_str("AppImage"));

        // Disable AUR and AppImage in settings
        scope.clamp_to_enabled(false, true, false);
        assert!(!scope.aur);
        assert!(!scope.appimage);
        assert!(scope.alpm);
        assert!(scope.flatpak);
        assert_eq!(scope.enabled_sources(false, true, false).len(), 2);
        assert!(scope.contains_str("Standard"));
        assert!(!scope.contains_str("AUR"));
        assert!(scope.contains_str("Flatpak"));
        assert!(!scope.contains_str("AppImage"));

        // Toggle source
        let mut scope = SourceScope::all();
        scope.toggle_source(PackageSourceKind::Aur);
        assert!(!scope.aur);
        assert!(scope.alpm);
        assert!(scope.flatpak);
        assert!(scope.appimage);
        assert_eq!(scope.enabled_sources(true, true, true).len(), 3);

        // The three inventory ecosystems have their own flag: a Cargo row no
        // longer disappears with ALPM, and the control offers them in the
        // scopes that list them.
        assert!(scope.is_selected(PackageSourceKind::Cargo));
        assert!(scope.is_selected(PackageSourceKind::Npm));
        assert!(scope.is_selected(PackageSourceKind::Release));
        scope.toggle_source(PackageSourceKind::Alpm);
        assert!(!scope.contains_str("ALPM"));
        assert!(scope.contains_str("Cargo"));
        assert!(scope.contains_str("NPM"));

        // The scope decides what the control offers.
        let results = crate::state::sources::scope_kinds(NavScope::Results);
        assert_eq!(results.len(), 4);
        assert!(!results.contains(&PackageSourceKind::Cargo));

        let installed = crate::state::sources::scope_kinds(NavScope::Installed);
        assert_eq!(installed.len(), 7);
        // The stores come before the installation methods.
        assert_eq!(
            installed
                .iter()
                .filter(|kind| kind.family() == crate::state::sources::SourceFamily::Store)
                .count(),
            5
        );

        // The selected sources of a scope are the scope's, among the selected.
        let selected = scope.sources_for(NavScope::Installed, true, true, true);
        assert!(selected.contains(&PackageSourceKind::Cargo));
        assert!(!selected.contains(&PackageSourceKind::Alpm));
    }

    #[test]
    fn the_query_narrows_the_rows_of_the_scope() {
        use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};

        let package = UnifiedPackage {
            name: "ripgrep".into(),
            version: "14.1.1".into(),
            description: "A search tool".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "repos".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        assert!(matches_query(&package, ""));
        assert!(matches_query(&package, "rip"));
        assert!(matches_query(&package, "  RIPGREP  "));
        assert!(matches_query(&package, "search tool"));
        assert!(!matches_query(&package, "firefox"));

        let other = UnifiedPackage {
            name: "firefox".into(),
            version: "128.0".into(),
            description: "A web browser".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "repos".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        assert!(!matches_query(&other, "ripgrep"));
    }

    #[test]
    fn test_query_workbench_breakpoints_wide_medium_narrow() {
        assert_eq!(
            WorkbenchBreakpoint::from_width(800.0),
            WorkbenchBreakpoint::Wide
        );
        assert_eq!(
            WorkbenchBreakpoint::from_width(640.0),
            WorkbenchBreakpoint::Wide
        );
        assert_eq!(
            WorkbenchBreakpoint::from_width(639.0),
            WorkbenchBreakpoint::Medium
        );
        assert_eq!(
            WorkbenchBreakpoint::from_width(460.0),
            WorkbenchBreakpoint::Medium
        );
        assert_eq!(
            WorkbenchBreakpoint::from_width(459.0),
            WorkbenchBreakpoint::Narrow
        );
        assert_eq!(
            WorkbenchBreakpoint::from_width(340.0),
            WorkbenchBreakpoint::Narrow
        );
    }
}
