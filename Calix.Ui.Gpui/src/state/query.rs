use crate::backend::models::UnifiedPackage;
use crate::state::session::PackageSourceKind;
use serde::{Deserialize, Serialize};

/// Portée de filtrage des sources (sélection multiple ou unitaire de dépôts)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceScope {
    pub alpm: bool,
    pub aur: bool,
    pub flatpak: bool,
    pub appimage: bool,
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
        }
    }

    pub fn all_enabled(aur_enabled: bool, flatpak_enabled: bool, appimage_enabled: bool) -> Self {
        let mut s = Self::all();
        s.clamp_to_enabled(aur_enabled, flatpak_enabled, appimage_enabled);
        s
    }

    pub fn is_all(&self) -> bool {
        self.alpm && self.aur && self.flatpak && self.appimage
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

    pub fn toggle_source(&mut self, source: PackageSourceKind) {
        match source {
            PackageSourceKind::Alpm => {
                if !self.alpm || (self.aur || self.flatpak || self.appimage) {
                    self.alpm = !self.alpm;
                }
            }
            PackageSourceKind::Aur => {
                if !self.aur || (self.alpm || self.flatpak || self.appimage) {
                    self.aur = !self.aur;
                }
            }
            PackageSourceKind::Flatpak => {
                if !self.flatpak || (self.alpm || self.aur || self.appimage) {
                    self.flatpak = !self.flatpak;
                }
            }
            PackageSourceKind::AppImage => {
                if !self.appimage || (self.alpm || self.aur || self.flatpak) {
                    self.appimage = !self.appimage;
                }
            }
            // Les écosystèmes Calix ne sont pas des sources de recherche
            // interrogeables : ils ne sont pas activables dans la requête.
            PackageSourceKind::Cargo | PackageSourceKind::Npm | PackageSourceKind::Release => {}
        }
        // Invariant strict : interdiction absolue d'avoir zéro source sélectionnée
        if !self.alpm && !self.aur && !self.flatpak && !self.appimage {
            self.alpm = true;
        }
    }

    pub fn contains_str(&self, source_type: &str) -> bool {
        match source_type {
            "Standard" | "Official" | "ALPM" => self.alpm,
            "AUR" => self.aur,
            "Flatpak" => self.flatpak,
            "AppImage" => self.appimage,
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
        if !self.alpm && !self.aur && !self.flatpak && !self.appimage {
            self.alpm = true;
        }
    }

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

        // Cannot deselect all sources down to 0
        scope.toggle_source(PackageSourceKind::Alpm);
        scope.toggle_source(PackageSourceKind::Flatpak);
        assert_eq!(scope.enabled_sources(true, true, true).len(), 1);
        assert!(scope.appimage);
        // Attempting to deselect the last remaining source (AppImage) is refused (never 0 sources)
        scope.toggle_source(PackageSourceKind::AppImage);
        assert_eq!(scope.enabled_sources(true, true, true).len(), 1);
        assert!(scope.appimage);
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
