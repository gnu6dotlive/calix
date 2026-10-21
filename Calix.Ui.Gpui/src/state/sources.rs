//! The source axis: what the seven sources are, what they hold, what they
//! cannot do, and where else a package is found.
//!
//! The sources are not interchangeable, and the interface says so before the
//! user asks. A source is either a **store** — a package manager the user
//! goes to — or an **installation method** — a single binary that arrives on
//! its own. The two are not the same kind of thing, and the axis keeps them
//! apart: the stores first, the methods last, and every entry states what it
//! holds.
//!
//! The absence of an action expresses a limit, and the reason lives in the
//! source's own entry. A boolean says "no"; a reason says why.

use crate::backend::models::UnifiedPackage;
use crate::state::query::SourceScope;
use crate::state::session::{NavScope, PackageSourceKind};

/// What kind of thing a source is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFamily {
    /// A package manager the user goes to for software: the stores.
    Store,
    /// A single self-contained binary that arrives on its own: the methods.
    Method,
}

impl PackageSourceKind {
    /// The family of the source: a store, or an installation method.
    pub fn family(self) -> SourceFamily {
        match self {
            PackageSourceKind::Alpm
            | PackageSourceKind::Aur
            | PackageSourceKind::Flatpak
            | PackageSourceKind::Cargo
            | PackageSourceKind::Npm => SourceFamily::Store,
            // An AppImage and a followed release are not stores: the software
            // is a single file, and it comes from where it was built.
            PackageSourceKind::AppImage | PackageSourceKind::Release => SourceFamily::Method,
        }
    }

    /// What the source holds, in the words the interface uses.
    pub fn holds(self) -> &'static str {
        match self {
            PackageSourceKind::Alpm => "official repositories — binaries",
            PackageSourceKind::Aur => "build recipes, not binaries",
            PackageSourceKind::Flatpak => "sandboxed applications",
            PackageSourceKind::Cargo => "crates from crates.io",
            PackageSourceKind::Npm => "global node packages",
            PackageSourceKind::AppImage => "a single self-contained binary",
            PackageSourceKind::Release => "binaries followed from a git host",
        }
    }

    /// The name of the source, short.
    pub fn short_label(self) -> &'static str {
        match self {
            PackageSourceKind::Alpm => "ALPM",
            PackageSourceKind::Aur => "AUR",
            PackageSourceKind::Flatpak => "Flatpak",
            PackageSourceKind::Cargo => "Cargo",
            PackageSourceKind::Npm => "NPM",
            PackageSourceKind::AppImage => "AppImage",
            PackageSourceKind::Release => "Tracked",
        }
    }
}

/// The kind of a source, read from the package's own source type.
pub fn kind_of_source_type(source_type: &str) -> PackageSourceKind {
    match source_type {
        "AUR" => PackageSourceKind::Aur,
        "Flatpak" => PackageSourceKind::Flatpak,
        "AppImage" => PackageSourceKind::AppImage,
        "Cargo" => PackageSourceKind::Cargo,
        "NPM" => PackageSourceKind::Npm,
        "Release" => PackageSourceKind::Release,
        // "ALPM", "Standard", "Official" and anything unknown: the official
        // repositories, which is where Calix looks first.
        _ => PackageSourceKind::Alpm,
    }
}

/// What a source cannot do, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLimit {
    /// The engine does not manage this source: there is no command to run.
    NotManaged,
    /// The source holds no dependencies: there is nothing to show.
    NoDependencies,
}

impl SourceLimit {
    /// The reason, in one line, for the user who wonders.
    pub fn reason(self) -> &'static str {
        match self {
            SourceLimit::NotManaged => "Calix does not manage this source: no command is available",
            SourceLimit::NoDependencies => "This source holds no dependencies",
        }
    }
}

/// The limits of a source, in the source's own entry.
///
/// A binary repository has no recipe to build; a recipe repository has no
/// binary to update; a sandbox has no dependencies to show; an ecosystem the
/// engine does not manage has no command at all. The limits that do not
/// apply are not written down: an empty list is a source that can do
/// everything the product asks of it.
pub fn limits_of(kind: PackageSourceKind) -> &'static [SourceLimit] {
    match kind {
        PackageSourceKind::Alpm | PackageSourceKind::Aur => &[],
        PackageSourceKind::Flatpak => &[SourceLimit::NoDependencies],
        PackageSourceKind::AppImage
        | PackageSourceKind::Cargo
        | PackageSourceKind::Npm
        | PackageSourceKind::Release => &[SourceLimit::NotManaged, SourceLimit::NoDependencies],
    }
}

/// How many rows each source holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SourceCounts {
    pub alpm: usize,
    pub aur: usize,
    pub flatpak: usize,
    pub appimage: usize,
    pub cargo: usize,
    pub npm: usize,
    pub release: usize,
}

impl SourceCounts {
    /// The count of one source.
    pub fn of(&self, kind: PackageSourceKind) -> usize {
        match kind {
            PackageSourceKind::Alpm => self.alpm,
            PackageSourceKind::Aur => self.aur,
            PackageSourceKind::Flatpak => self.flatpak,
            PackageSourceKind::AppImage => self.appimage,
            PackageSourceKind::Cargo => self.cargo,
            PackageSourceKind::Npm => self.npm,
            PackageSourceKind::Release => self.release,
        }
    }
}

/// Counts the rows of a scope by source.
pub fn counts_by_source(packages: &[UnifiedPackage]) -> SourceCounts {
    let mut counts = SourceCounts::default();
    for package in packages {
        match kind_of_source_type(&package.source_type) {
            PackageSourceKind::Alpm => counts.alpm += 1,
            PackageSourceKind::Aur => counts.aur += 1,
            PackageSourceKind::Flatpak => counts.flatpak += 1,
            PackageSourceKind::AppImage => counts.appimage += 1,
            PackageSourceKind::Cargo => counts.cargo += 1,
            PackageSourceKind::Npm => counts.npm += 1,
            PackageSourceKind::Release => counts.release += 1,
        }
    }
    counts
}

/// The sources a scope can hold, in the order of the families.
pub fn scope_kinds(scope: NavScope) -> Vec<PackageSourceKind> {
    scope
        .sources()
        .iter()
        .map(|source| match *source {
            "alpm" => PackageSourceKind::Alpm,
            "aur" => PackageSourceKind::Aur,
            "flatpak" => PackageSourceKind::Flatpak,
            "appimage" => PackageSourceKind::AppImage,
            "cargo" => PackageSourceKind::Cargo,
            "npm" => PackageSourceKind::Npm,
            _ => PackageSourceKind::Release,
        })
        .collect()
}

/// The name of a source, as the control presents it.
pub fn source_label(kind: PackageSourceKind) -> &'static str {
    match kind {
        PackageSourceKind::Alpm => "Official / ALPM",
        PackageSourceKind::Aur => "AUR",
        PackageSourceKind::Flatpak => "Flatpak",
        PackageSourceKind::AppImage => "AppImage",
        PackageSourceKind::Cargo => "Cargo",
        PackageSourceKind::Npm => "NPM",
        PackageSourceKind::Release => "Tracked releases",
    }
}

/// An entry of the source axis, as the control presents it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceAxisEntry {
    pub kind: PackageSourceKind,
    pub family: SourceFamily,
    pub label: &'static str,
    /// What the source holds, stated before the user asks.
    pub holds: &'static str,
    /// How many rows of the current scope the source holds.
    pub count: usize,
    /// The source is listed but never queried.
    pub inventory_only: bool,
    /// The source is selected in the current filter.
    pub selected: bool,
}

/// The source axis, as the control presents it.
///
/// Every source the scope can hold, in the order of the families — the stores
/// first, the installation methods last — with what each one holds and how
/// many rows it has. The scope decides what is offered; the filter decides
/// what is selected; the order is the source's own.
pub fn source_axis(
    scope: NavScope,
    packages: &[UnifiedPackage],
    filter: &SourceScope,
) -> Vec<SourceAxisEntry> {
    let counts = counts_by_source(packages);
    scope_kinds(scope)
        .into_iter()
        .map(|kind| SourceAxisEntry {
            kind,
            family: kind.family(),
            label: source_label(kind),
            holds: kind.holds(),
            count: counts.of(kind),
            inventory_only: scope
                .inventory_only_sources()
                .contains(&kind.as_str()),
            selected: filter.is_selected(kind),
        })
        .collect()
}

/// L'ancre de la banque : le canal qui porte le plus de lignes du périmètre.
///
/// La banque a un devant. À égalité, le premier l'emporte — au repos, quand
/// toutes les sources sont vides, la banque montre la même face, au lieu de
/// changer d'ancre au gré de l'ordre de l'inventaire.
pub fn anchor_index(axis: &[SourceAxisEntry]) -> usize {
    let mut best = 0;
    for (index, entry) in axis.iter().enumerate() {
        if entry.count > axis[best].count {
            best = index;
        }
    }
    best
}

/// The same package, in another source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlsoIn {
    pub source: PackageSourceKind,
    pub version: String,
    pub installed: bool,
}

/// The other sources where the same name is found.
///
/// A name is not a package: `ripgrep` is a binary in the official
/// repositories, a recipe in the AUR, and a crate on crates.io. The card says
/// so, and says where else the name is found — the sources are not
/// interchangeable, and the same name in two of them is two different
/// things.
pub fn also_in(package: &UnifiedPackage, packages: &[UnifiedPackage]) -> Vec<AlsoIn> {
    let name = package.name.to_lowercase();
    let own = kind_of_source_type(&package.source_type);
    let mut found: Vec<AlsoIn> = Vec::new();
    for other in packages {
        if other.name.to_lowercase() != name {
            continue;
        }
        let source = kind_of_source_type(&other.source_type);
        if source == own {
            continue;
        }
        if found.iter().any(|entry| entry.source == source) {
            continue;
        }
        found.push(AlsoIn {
            source,
            version: other.version.clone(),
            installed: other.is_installed,
        });
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::models::{AlpmPackage, UnifiedPackageSource};

    fn alpm(name: &str, version: &str) -> UnifiedPackage {
        let inner = AlpmPackage {
            name: name.to_string(),
            version: version.to_string(),
            ..Default::default()
        };
        UnifiedPackage {
            name: name.to_string(),
            version: version.to_string(),
            description: String::new(),
            source_type: "ALPM".to_string(),
            repository_or_remote: "extra".to_string(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(inner),
        }
    }

    fn from_source(name: &str, source_type: &str, version: &str) -> UnifiedPackage {
        let mut package = alpm(name, version);
        package.source_type = source_type.to_string();
        package
    }

    #[test]
    fn the_stores_come_before_the_installation_methods() {
        // The axis keeps the two kinds apart: a store is a package manager the
        // user goes to; a method is a single binary that arrives on its own.
        let stores: Vec<PackageSourceKind> = PackageSourceKind::ALL
            .iter()
            .copied()
            .filter(|kind| kind.family() == SourceFamily::Store)
            .collect();
        let methods: Vec<PackageSourceKind> = PackageSourceKind::ALL
            .iter()
            .copied()
            .filter(|kind| kind.family() == SourceFamily::Method)
            .collect();
        assert_eq!(stores.len(), 5);
        assert_eq!(methods.len(), 2);
        // The methods are the two that are not a package manager.
        assert!(methods.contains(&PackageSourceKind::AppImage));
        assert!(methods.contains(&PackageSourceKind::Release));
        // And they come last: the rare ones are grouped.
        let last_two: Vec<PackageSourceKind> =
            PackageSourceKind::ALL[PackageSourceKind::ALL.len() - 2..].to_vec();
        assert_eq!(last_two, methods);
    }

    #[test]
    fn every_source_states_what_it_holds() {
        for kind in PackageSourceKind::ALL {
            assert!(
                !kind.holds().is_empty(),
                "{} must say what it holds",
                kind.short_label()
            );
        }
        assert_eq!(
            PackageSourceKind::Aur.holds(),
            "build recipes, not binaries"
        );
    }

    #[test]
    fn the_limits_of_a_source_are_the_reasons_why_an_action_is_absent() {
        // A store the engine manages has no limit to state.
        assert!(limits_of(PackageSourceKind::Alpm).is_empty());
        assert!(limits_of(PackageSourceKind::Aur).is_empty());
        // A sandbox holds no dependencies to show.
        assert!(limits_of(PackageSourceKind::Flatpak).contains(&SourceLimit::NoDependencies));
        // An ecosystem the engine does not manage has no command at all.
        assert!(limits_of(PackageSourceKind::Cargo).contains(&SourceLimit::NotManaged));
        assert!(limits_of(PackageSourceKind::Release).contains(&SourceLimit::NotManaged));
        // Every limit says why.
        for kind in PackageSourceKind::ALL {
            for limit in limits_of(*kind) {
                assert!(!limit.reason().is_empty());
            }
        }
    }

    #[test]
    fn the_counts_follow_the_rows() {
        let packages = vec![
            alpm("ripgrep", "14.1.0"),
            from_source("ripgrep", "AUR", "14.1.0"),
            from_source("ripgrep", "Cargo", "14.1.0"),
            alpm("fd", "9.0.0"),
        ];
        let counts = counts_by_source(&packages);
        assert_eq!(counts.alpm, 2);
        assert_eq!(counts.aur, 1);
        assert_eq!(counts.cargo, 1);
        assert_eq!(counts.of(PackageSourceKind::AppImage), 0);
    }

    #[test]
    fn the_axis_offers_what_the_scope_can_hold_and_counts_it() {
        let packages = vec![alpm("ripgrep", "14.1.0"), from_source("ripgrep", "AUR", "14.1.0")];
        let filter = SourceScope::all();
        let axis = source_axis(NavScope::Results, &packages, &filter);
        // The results scope holds four searchable sources.
        assert_eq!(axis.len(), 4);
        assert!(axis.iter().all(|entry| entry.selected));
        assert!(axis.iter().all(|entry| !entry.inventory_only));
        let alpm = axis
            .iter()
            .find(|entry| entry.kind == PackageSourceKind::Alpm)
            .expect("ALPM is offered");
        assert_eq!(alpm.count, 1);
        assert_eq!(alpm.family, SourceFamily::Store);

        // The installed scope holds the three ecosystems too, and they are
        // marked as listed rather than queried.
        let axis = source_axis(NavScope::Installed, &packages, &filter);
        assert_eq!(axis.len(), 7);
        let cargo = axis
            .iter()
            .find(|entry| entry.kind == PackageSourceKind::Cargo)
            .expect("Cargo is offered");
        assert!(cargo.inventory_only);
    }

    #[test]
    fn the_axis_follows_the_filter() {
        let packages = vec![alpm("ripgrep", "14.1.0")];
        let mut filter = SourceScope::all();
        filter.toggle_source(PackageSourceKind::Aur);
        let axis = source_axis(NavScope::Results, &packages, &filter);
        let aur = axis
            .iter()
            .find(|entry| entry.kind == PackageSourceKind::Aur)
            .expect("AUR is offered");
        assert!(!aur.selected);
        let alpm = axis
            .iter()
            .find(|entry| entry.kind == PackageSourceKind::Alpm)
            .expect("ALPM is offered");
        assert!(alpm.selected);
    }

    #[test]
    fn the_bank_is_anchored_on_the_channel_that_carries_the_most() {
        let packages = vec![
            alpm("ripgrep", "14.1.0"),
            alpm("fd", "9.0.0"),
            from_source("ripgrep", "AUR", "14.1.0-1"),
        ];
        let axis = source_axis(NavScope::Results, &packages, &SourceScope::all());
        let anchor = &axis[anchor_index(&axis)];
        assert_eq!(anchor.kind, PackageSourceKind::Alpm);
        assert_eq!(anchor.count, 2);

        // À égalité, le premier l'emporte : la banque ne change pas de face
        // quand deux canaux portent autant.
        let tie = vec![
            alpm("ripgrep", "14.1.0"),
            from_source("ripgrep", "AUR", "14.1.0-1"),
        ];
        let axis = source_axis(NavScope::Results, &tie, &SourceScope::all());
        assert_eq!(axis[anchor_index(&axis)].kind, PackageSourceKind::Alpm);

        // Au repos, la banque garde la même ancre.
        let axis = source_axis(NavScope::Results, &[], &SourceScope::all());
        assert_eq!(axis[anchor_index(&axis)].kind, PackageSourceKind::Alpm);
    }

    #[test]
    fn the_card_says_where_else_the_name_is_found() {
        let packages = vec![
            alpm("ripgrep", "14.1.0"),
            from_source("ripgrep", "AUR", "14.1.0-1"),
            from_source("ripgrep", "Cargo", "14.1.0"),
            alpm("fd", "9.0.0"),
        ];
        let found = also_in(&packages[0], &packages);
        assert_eq!(found.len(), 2);
        assert!(found.iter().any(|entry| entry.source == PackageSourceKind::Aur));
        assert!(found.iter().any(|entry| entry.source == PackageSourceKind::Cargo));
        // A name that is alone is alone.
        let found = also_in(&packages[3], &packages);
        assert!(found.is_empty());
    }
}
