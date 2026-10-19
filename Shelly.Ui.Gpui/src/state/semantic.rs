use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};
use serde::{Deserialize, Serialize};

/// Catégorie typée de relation de dépendance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DependencyKind {
    Runtime,
    Optional,
    Make,
    Conflict,
    Provides,
    RequiredBy,
    OptionalFor,
}

impl DependencyKind {
    pub fn label(&self) -> &'static str {
        match self {
            DependencyKind::Runtime => "Runtime Dependencies",
            DependencyKind::Optional => "Optional Dependencies",
            DependencyKind::Make => "Build / Make Dependencies",
            DependencyKind::Conflict => "Conflicts",
            DependencyKind::Provides => "Provides",
            DependencyKind::RequiredBy => "Required By",
            DependencyKind::OptionalFor => "Optional For",
        }
    }
}

/// Référence typée et analysée vers une dépendance de paquet
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyRef {
    pub raw: String,
    pub name: String,
    pub constraint: Option<String>,
    pub description: Option<String>,
    pub kind: DependencyKind,
    pub is_navigable: bool,
}

impl DependencyRef {
    /// Analyse une chaîne brute de dépendance (ex: "glibc>=2.34", "libxkbfile: for keyboard shortcuts")
    pub fn parse(raw: &str, kind: DependencyKind) -> Self {
        let raw_trimmed = raw.trim();
        let (pkg_part, description) = if let Some((left, right)) = raw_trimmed.split_once(':') {
            (left.trim(), Some(right.trim().to_string()))
        } else {
            (raw_trimmed, None)
        };

        // Recherche d'opérateurs de version : >=, <=, =, >, <
        let op_pos = [">=", "<=", "=", ">", "<"]
            .iter()
            .filter_map(|op| pkg_part.find(op).map(|idx| (idx, *op)))
            .min_by_key(|(idx, _)| *idx);

        let (name, constraint) = if let Some((idx, _op)) = op_pos {
            let n = pkg_part[..idx].trim().to_string();
            let c = pkg_part[idx..].trim().to_string();
            (n, Some(c))
        } else {
            (pkg_part.trim().to_string(), None)
        };

        let is_navigable = !name.is_empty() && !name.contains('/');

        Self {
            raw: raw_trimmed.to_string(),
            name,
            constraint,
            description,
            kind,
            is_navigable,
        }
    }
}

/// Cible sémantique d'interaction au clic
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticTarget {
    Package(String),
    ExternalUrl(String),
    FilePath(String),
    CopyText(String),
}

/// Matrice des capacités fonctionnelles dérivées pour un paquet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageCapabilities {
    pub can_install: bool,
    pub can_remove: bool,
    pub can_update: bool,
    pub has_dependencies: bool,
    pub has_upstream_url: bool,
    pub has_local_path: bool,
    pub has_build_metadata: bool,
}

impl PackageCapabilities {
    pub fn derive(pkg: &UnifiedPackage, alpm_detail: Option<&AlpmPackage>) -> Self {
        let is_appimage = pkg.source_type == "AppImage";
        let is_aur = pkg.source_type == "AUR";
        let is_flatpak = pkg.source_type == "Flatpak";
        // Écosystèmes maintenus par le moteur Calix : ils ne passent pas par
        // le backend Shelly, donc aucune commande d'installation ou de
        // suppression n'est disponible pour eux.
        let is_ecosystem = matches!(pkg.source_type.as_str(), "Cargo" | "NPM" | "Release");
        let is_alpm = pkg.source_type == "ALPM"
            || (!is_appimage && !is_aur && !is_flatpak && !is_ecosystem);

        let can_install = !pkg.is_installed && !is_appimage && !is_ecosystem;
        let can_remove = pkg.is_installed && !is_appimage && !is_ecosystem;
        let can_update = pkg.has_update;

        let has_dependencies = is_alpm || is_aur;

        let has_upstream_url = match &pkg.inner {
            UnifiedPackageSource::Standard(a) => a
                .url
                .as_ref()
                .map(|u| !u.trim().is_empty())
                .unwrap_or(false),
            UnifiedPackageSource::Aur(a) => a
                .url
                .as_ref()
                .map(|u| !u.trim().is_empty())
                .unwrap_or(false),
            UnifiedPackageSource::Flatpak(_) => true,
            UnifiedPackageSource::AppImage(ai) => ai
                .update_url
                .as_ref()
                .map(|u| !u.trim().is_empty())
                .unwrap_or(false),
            UnifiedPackageSource::Cargo(_) => false,
            UnifiedPackageSource::Npm(_) => false,
            UnifiedPackageSource::Release(r) => r
                .repo
                .as_ref()
                .map(|repo| match r.provider.as_deref() {
                    Some("github") => format!("https://github.com/{}", repo),
                    Some("gitlab") => format!("https://gitlab.com/{}", repo),
                    _ => repo.clone(),
                })
                .map(|url| !url.trim().is_empty())
                .unwrap_or(false),
        } || alpm_detail
            .and_then(|a| a.url.as_ref())
            .map(|u| !u.trim().is_empty())
            .unwrap_or(false);

        let has_local_path = match &pkg.inner {
            UnifiedPackageSource::AppImage(ai) => ai.path.is_some(),
            _ => false,
        };

        let has_build_metadata = is_aur || is_alpm || is_appimage;

        Self {
            can_install,
            can_remove,
            can_update,
            has_dependencies,
            has_upstream_url,
            has_local_path,
            has_build_metadata,
        }
    }
}

/// Génère la commande d'installation canonique Shelly selon la source
pub fn canonical_install_command(pkg: &UnifiedPackage) -> Option<String> {
    match pkg.source_type.as_str() {
        "AUR" => Some(format!("shelly install aur {}", pkg.name)),
        "Flatpak" => Some(format!("shelly install flatpak {}", pkg.name)),
        "ALPM" => Some(format!("shelly install standard {}", pkg.name)),
        "AppImage" => None,
        // Les écosystèmes Calix ne sont pas gérés par le backend Shelly :
        // aucune commande d'installation n'existe pour eux.
        "Cargo" | "NPM" | "Release" => None,
        _ => Some(format!("shelly install standard {}", pkg.name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::calix_state::{CargoCrate, NpmPackage, ReleasePackage};
    use core::prelude::v1::test;

    #[test]
    fn test_dependency_ref_parse_versioned() {
        let dep1 = DependencyRef::parse("glibc>=2.34", DependencyKind::Runtime);
        assert_eq!(dep1.name, "glibc");
        assert_eq!(dep1.constraint, Some(">=2.34".into()));
        assert_eq!(dep1.description, None);
        assert!(dep1.is_navigable);

        let dep2 = DependencyRef::parse("python=3.12", DependencyKind::Make);
        assert_eq!(dep2.name, "python");
        assert_eq!(dep2.constraint, Some("=3.12".into()));
        assert_eq!(dep2.description, None);

        let dep3 = DependencyRef::parse("gcc-libs<15", DependencyKind::Conflict);
        assert_eq!(dep3.name, "gcc-libs");
        assert_eq!(dep3.constraint, Some("<15".into()));
    }

    #[test]
    fn test_dependency_ref_parse_with_description() {
        let dep = DependencyRef::parse(
            "libxkbfile: for keyboard shortcuts",
            DependencyKind::Optional,
        );
        assert_eq!(dep.name, "libxkbfile");
        assert_eq!(dep.constraint, None);
        assert_eq!(dep.description, Some("for keyboard shortcuts".into()));
        assert!(dep.is_navigable);

        let dep_both = DependencyRef::parse(
            "python-numpy>=1.20: fast math support",
            DependencyKind::Optional,
        );
        assert_eq!(dep_both.name, "python-numpy");
        assert_eq!(dep_both.constraint, Some(">=1.20".into()));
        assert_eq!(dep_both.description, Some("fast math support".into()));
    }

    #[test]
    fn test_dependency_ref_parse_unversioned() {
        let dep = DependencyRef::parse("curl", DependencyKind::Runtime);
        assert_eq!(dep.name, "curl");
        assert_eq!(dep.constraint, None);
        assert_eq!(dep.description, None);
        assert!(dep.is_navigable);
    }

    #[test]
    fn test_canonical_install_command() {
        let mut pkg = UnifiedPackage {
            name: "ripgrep".into(),
            version: "14.1.0".into(),
            description: "search tool".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        assert_eq!(
            canonical_install_command(&pkg),
            Some("shelly install standard ripgrep".into())
        );

        pkg.source_type = "AUR".into();
        assert_eq!(
            canonical_install_command(&pkg),
            Some("shelly install aur ripgrep".into())
        );

        pkg.source_type = "Flatpak".into();
        assert_eq!(
            canonical_install_command(&pkg),
            Some("shelly install flatpak ripgrep".into())
        );

        pkg.source_type = "AppImage".into();
        assert_eq!(canonical_install_command(&pkg), None);

        // Les écosystèmes Calix ne passent pas par le backend Shelly.
        for source in ["Cargo", "NPM", "Release"] {
            pkg.source_type = source.into();
            assert_eq!(
                canonical_install_command(&pkg),
                None,
                "{} must not produce a Shelly install command",
                source
            );
        }
    }

    /// Un paquet d'un écosystème Calix ne doit jamais être classé comme ALPM.
    ///
    /// `is_alpm` repose sur un repli « toute source inconnue est ALPM ».
    /// Sans l'exclusion explicite des écosystèmes Calix, une crate Cargo
    /// héritait des capacités ALPM (dépendances, build, suppression).
    #[test]
    fn test_ecosystem_packages_are_not_classified_as_alpm() {
        for (source, inner) in [
            (
                "Cargo",
                UnifiedPackageSource::Cargo(CargoCrate {
                    version: "14.1.1".into(),
                    source: "crates.io".into(),
                    bins: vec!["rg".into()],
                }),
            ),
            (
                "NPM",
                UnifiedPackageSource::Npm(NpmPackage {
                    version: "5.6.3".into(),
                    overridden: false,
                }),
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
            ),
        ] {
            let pkg = UnifiedPackage {
                name: "ecosystem-pkg".into(),
                version: "1.0.0".into(),
                description: "test".into(),
                source_type: source.into(),
                repository_or_remote: "origin".into(),
                is_installed: true,
                has_update: false,
                new_version: None,
                inner,
            };

            let caps = PackageCapabilities::derive(&pkg, None);
            assert!(
                !caps.has_dependencies,
                "{} must not expose dependency resolution",
                source
            );
            assert!(
                !caps.can_install,
                "{} must not offer an install action",
                source
            );
            assert!(
                !caps.can_remove,
                "{} must not offer a remove action",
                source
            );
        }
    }

    /// Une release suivie par Calix expose son URL amont et son état de mise à jour.
    #[test]
    fn test_release_package_exposes_upstream_url_and_update_state() {
        let pkg = UnifiedPackage {
            name: "artcraft-launcher".into(),
            version: "0.5.1".into(),
            description: "test".into(),
            source_type: "Release".into(),
            repository_or_remote: "storytold/craft-launcher".into(),
            is_installed: true,
            has_update: true,
            new_version: Some("0.5.2".into()),
            inner: UnifiedPackageSource::Release(ReleasePackage {
                installed_version: Some("0.5.1".into()),
                latest_version: Some("0.5.2".into()),
                update_available: true,
                repo: Some("storytold/craft-launcher".into()),
                provider: Some("github".into()),
                canal: Some("stable".into()),
                pkgname: Some("artcraft-launcher-bin".into()),
            }),
        };

        let caps = PackageCapabilities::derive(&pkg, None);
        assert!(caps.has_upstream_url);
        assert!(caps.can_update);
    }

    #[test]
    fn test_package_capabilities_derive() {
        let pkg = UnifiedPackage {
            name: "test-pkg".into(),
            version: "1.0.0".into(),
            description: "test".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: false,
            has_update: true,
            new_version: Some("1.1.0".into()),
            inner: UnifiedPackageSource::Standard(AlpmPackage {
                url: Some("https://archlinux.org".into()),
                ..Default::default()
            }),
        };

        let caps = PackageCapabilities::derive(&pkg, None);
        assert!(caps.can_install);
        assert!(!caps.can_remove);
        assert!(caps.can_update);
        assert!(caps.has_dependencies);
        assert!(caps.has_upstream_url);
        assert!(!caps.has_local_path);
        assert!(caps.has_build_metadata);
    }
}
