use serde::{Deserialize, Serialize};

/// Représente un paquet ALPM standard (dépôts officiels ou local)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AlpmPackage {
    pub name: String,
    #[serde(default)]
    pub package_base: Option<String>,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub licenses: Vec<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub opt_depends: Vec<String>,
    #[serde(default)]
    pub opt_depends_installed: Vec<bool>,
    #[serde(default)]
    pub conflicts: Vec<String>,
    #[serde(default)]
    pub install_reason: Option<String>,
    #[serde(default)]
    pub install_date: Option<String>,
    #[serde(default)]
    pub build_date: Option<String>,
    #[serde(default)]
    pub download_size: Option<u64>,
    #[serde(default)]
    pub installed_size: Option<u64>,
    #[serde(default)]
    pub required_by: Vec<String>,
    #[serde(default)]
    pub optional_for: Vec<String>,
}

/// Représente un paquet AUR retourné par Shelly
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AurPackage {
    #[serde(default)]
    pub id: Option<u64>,
    pub name: String,
    #[serde(default)]
    pub package_base: Option<String>,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub num_votes: Option<u32>,
    #[serde(default)]
    pub popularity: Option<f64>,
    #[serde(default)]
    pub out_of_date: Option<u64>,
    #[serde(default)]
    pub maintainer: Option<String>,
    #[serde(default)]
    pub first_submitted: Option<u64>,
    #[serde(default)]
    pub last_modified: Option<u64>,
    #[serde(default)]
    pub depends: Option<Vec<String>>,
    #[serde(default)]
    pub make_depends: Option<Vec<String>>,
    #[serde(default)]
    pub opt_depends: Option<Vec<String>>,
    #[serde(default)]
    pub conflicts: Option<Vec<String>>,
    #[serde(default)]
    pub provides: Option<Vec<String>>,
    #[serde(default)]
    pub license: Option<Vec<String>>,
    #[serde(default)]
    pub keywords: Option<Vec<String>>,
}

/// Représente un hit de recherche Flatpak
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FlatpakHit {
    pub name: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type", default)]
    pub app_type: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub remote: Option<String>,
    #[serde(default)]
    pub download_size: Option<u64>,
    #[serde(default)]
    pub installed_size: Option<u64>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub main_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FlatpakSearchResult {
    pub hits: Vec<FlatpakHit>,
    #[serde(default)]
    pub query: String,
    #[serde(rename = "totalHits", default)]
    pub total_hits: usize,
}

/// Représente un élément d'actualité Arch Linux
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ArchNewsItem {
    pub title: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub published_date: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

/// Représente une mise à jour disponible retournée par `shelly list-updates all -j`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct PackageUpdateItem {
    pub name: String,
    #[serde(default)]
    pub old_version: String,
    #[serde(default)]
    pub new_version: String,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub package_type: Option<String>,
    #[serde(default)]
    pub download_size: Option<u64>,
}

/// Représentation d'une AppImage gérée localement par Shelly
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AppImageItem {
    pub name: String,
    #[serde(default)]
    pub desktop_name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub icon_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub size_on_disk: Option<u64>,
    #[serde(default)]
    pub update_url: Option<String>,
    #[serde(default)]
    pub repo_owner: Option<String>,
    #[serde(default)]
    pub repo_name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

use crate::backend::calix_snapshot::{Snapshot, SnapshotPackage};

/// A crate installed by `cargo install`, as the inventory reports it.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct CargoCrate {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub bins: Vec<String>,
}

/// A globally installed npm package, as the inventory reports it.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct NpmPackage {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub overridden: bool,
}

/// A package tracked by the Calix release backend.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ReleasePackage {
    #[serde(default)]
    pub installed_version: Option<String>,
    #[serde(default)]
    pub latest_version: Option<String>,
    #[serde(default)]
    pub update_available: bool,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub canal: Option<String>,
    #[serde(default)]
    pub pkgname: Option<String>,
}

/// Représentation unifiée d'un paquet affiché dans la liste GPUI
#[derive(Debug, Clone, PartialEq)]
pub enum UnifiedPackageSource {
    Standard(AlpmPackage),
    Aur(AurPackage),
    Flatpak(FlatpakHit),
    AppImage(AppImageItem),
    Cargo(CargoCrate),
    Npm(NpmPackage),
    Release(ReleasePackage),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnifiedPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source_type: String,
    pub repository_or_remote: String,
    pub is_installed: bool,
    pub has_update: bool,
    pub new_version: Option<String>,
    pub inner: UnifiedPackageSource,
}

impl UnifiedPackage {
    pub fn from_alpm(pkg: AlpmPackage, is_installed: bool) -> Self {
        let repo = pkg
            .repository
            .clone()
            .unwrap_or_else(|| "repos".to_string());
        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            description: pkg.description.clone().unwrap_or_default(),
            source_type: "ALPM".to_string(),
            repository_or_remote: repo,
            is_installed,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(pkg),
        }
    }

    pub fn from_aur(pkg: AurPackage, is_installed: bool) -> Self {
        Self {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            description: pkg.description.clone().unwrap_or_default(),
            source_type: "AUR".to_string(),
            repository_or_remote: "aur".to_string(),
            is_installed,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Aur(pkg),
        }
    }

    pub fn from_flatpak(hit: FlatpakHit, is_installed: bool) -> Self {
        let remote = hit.remote.clone().unwrap_or_else(|| "flathub".to_string());
        Self {
            name: hit.name.clone(),
            version: hit.app_id.clone().unwrap_or_default(),
            description: hit.summary.clone().unwrap_or_default(),
            source_type: "Flatpak".to_string(),
            repository_or_remote: remote,
            is_installed,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Flatpak(hit),
        }
    }

    pub fn from_appimage(item: AppImageItem) -> Self {
        let desc = item.description.clone().unwrap_or_default();
        let ver = item
            .version
            .clone()
            .unwrap_or_else(|| "appimage".to_string());
        Self {
            name: item.name.clone(),
            version: ver,
            description: desc,
            source_type: "AppImage".to_string(),
            repository_or_remote: "appimage".to_string(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::AppImage(item),
        }
    }

    /// Construit un paquet unifié à partir d'une crate Cargo lue par l'inventaire Calix.
    pub fn from_cargo(name: String, crate_: CargoCrate) -> Self {
        let description = if crate_.bins.is_empty() {
            "Cargo crate".to_string()
        } else {
            format!("Cargo crate — binaries: {}", crate_.bins.join(", "))
        };
        let origin = if crate_.source.is_empty() {
            "crates.io".to_string()
        } else {
            crate_.source.clone()
        };
        Self {
            name,
            version: crate_.version.clone(),
            description,
            source_type: "Cargo".to_string(),
            repository_or_remote: origin,
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Cargo(crate_),
        }
    }

    /// Construit un paquet unifié à partir d'un paquet NPM global lu par l'inventaire Calix.
    pub fn from_npm(name: String, package: NpmPackage) -> Self {
        let description = if package.overridden {
            "NPM global package (version overridden)".to_string()
        } else {
            "NPM global package".to_string()
        };
        Self {
            name,
            version: package.version.clone(),
            description,
            source_type: "NPM".to_string(),
            repository_or_remote: "npm".to_string(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Npm(package),
        }
    }

    /// Construit un paquet unifié à partir d'un paquet suivi par le backend de releases Calix.
    pub fn from_release(name: String, release: ReleasePackage) -> Self {
        let installed = release
            .installed_version
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let latest = release.latest_version.clone();
        let has_update = release.update_available
            && latest
                .as_deref()
                .map(|value| value != installed)
                .unwrap_or(false);
        let origin = release
            .repo
            .clone()
            .unwrap_or_else(|| "release".to_string());
        let description = match (&release.provider, &release.canal) {
            (Some(provider), Some(canal)) => {
                format!("Release-managed package — {} channel, {}", provider, canal)
            }
            _ => "Release-managed package".to_string(),
        };
        Self {
            name,
            version: installed,
            description,
            source_type: "Release".to_string(),
            repository_or_remote: origin,
            is_installed: true,
            has_update,
            new_version: if has_update { latest } else { None },
            inner: UnifiedPackageSource::Release(release),
        }
    }

    /// Construit un paquet unifié à partir d'un enregistrement du daemon.
    ///
    /// ALPM et AUR sont volontairement ignorés. La liste typée des paquets
    /// pacman vient du backend CLI, qui porte les champs que l'inspecteur lit
    /// (dépendances, mainteneur, base du paquet) ; le document normalisé ne
    /// les a pas, et ses copies ALPM feraient doublon avec cette liste. AUR
    /// est un sous-ensemble d'ALPM : les mêmes paquets, deux fois.
    pub fn from_snapshot(package: &SnapshotPackage) -> Option<Self> {
        match package.source_type.as_str() {
            "Cargo" => Some(Self::from_cargo(
                package.name.clone(),
                CargoCrate {
                    version: package.version.clone(),
                    source: package.repository_or_remote.clone(),
                    bins: package.binaries.clone(),
                },
            )),
            "NPM" => Some(Self::from_npm(
                package.name.clone(),
                NpmPackage {
                    version: package.version.clone(),
                    overridden: package.overridden,
                },
            )),
            "Release" => Some(Self::from_release(
                package.name.clone(),
                ReleasePackage {
                    installed_version: Some(package.version.clone()),
                    latest_version: package.new_version.clone(),
                    update_available: package.has_update,
                    repo: non_empty(&package.repository_or_remote),
                    provider: package.provider.clone(),
                    canal: package.canal.clone(),
                    pkgname: package.pkgname.clone(),
                },
            )),
            _ => None,
        }
    }

    pub fn from_update(u: PackageUpdateItem) -> Self {
        Self {
            name: u.name,
            version: u.old_version,
            description: format!("Update available to {}", u.new_version),
            source_type: u.package_type.unwrap_or_else(|| "ALPM".to_string()),
            repository_or_remote: u.repository.unwrap_or_else(|| "repos".to_string()),
            is_installed: true,
            has_update: true,
            new_version: Some(u.new_version),
            inner: UnifiedPackageSource::Standard(Default::default()),
        }
    }

    pub fn key(&self) -> crate::state::PackageKey {
        crate::state::PackageKey::from_unified(self)
    }
}

/// Fusionne la liste typée du backend CLI avec ce que le daemon sert.
///
/// Le backend CLI reste la source des paquets ALPM ; le daemon apporte les
/// écosystèmes que pacman ne possède pas. C'est une fusion de deux origines,
/// plus de deux formats de stockage : le frontend ne lit plus aucun fichier.
pub fn merge_snapshot(alpm: Vec<AlpmPackage>, snapshot: &Snapshot) -> Vec<UnifiedPackage> {
    let mut packages: Vec<UnifiedPackage> = alpm
        .into_iter()
        .map(|pkg| UnifiedPackage::from_alpm(pkg, true))
        .collect();
    packages.extend(
        snapshot
            .packages
            .iter()
            .filter_map(UnifiedPackage::from_snapshot),
    );
    packages
}

fn non_empty(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(source_type: &str, name: &str, version: &str) -> SnapshotPackage {
        SnapshotPackage {
            name: name.to_string(),
            version: version.to_string(),
            source_type: source_type.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn a_cargo_record_keeps_its_binaries_and_registry() {
        let package = UnifiedPackage::from_snapshot(&SnapshotPackage {
            repository_or_remote: "registry+https://github.com/rust-lang/crates.io-index".into(),
            binaries: vec!["cargo-add".into(), "cargo-rm".into()],
            ..record("Cargo", "cargo-edit", "0.13.11")
        })
        .expect("a Cargo record becomes a package");

        assert_eq!(package.name, "cargo-edit");
        assert_eq!(package.version, "0.13.11");
        assert_eq!(package.source_type, "Cargo");
        assert!(package.description.contains("cargo-add, cargo-rm"));
        assert_eq!(package.repository_or_remote, "registry+https://github.com/rust-lang/crates.io-index");
    }

    #[test]
    fn an_npm_record_carries_the_overridden_flag() {
        let package = UnifiedPackage::from_snapshot(&SnapshotPackage {
            overridden: true,
            ..record("NPM", "typescript", "5.6.3")
        })
        .expect("an NPM record becomes a package");

        assert_eq!(package.source_type, "NPM");
        assert!(package.description.contains("overridden"));
        match package.inner {
            UnifiedPackageSource::Npm(npm) => assert!(npm.overridden),
            other => panic!("expected the NPM variant, got {other:?}"),
        }
    }

    #[test]
    fn a_release_record_carries_the_channel_and_the_newer_version() {
        let package = UnifiedPackage::from_snapshot(&SnapshotPackage {
            repository_or_remote: "storytold/craft-launcher".into(),
            has_update: true,
            new_version: Some("0.5.2".into()),
            provider: Some("github".into()),
            canal: Some("stable".into()),
            pkgname: Some("artcraft-launcher-bin".into()),
            ..record("Release", "artcraft-launcher", "0.5.1")
        })
        .expect("a Release record becomes a package");

        assert_eq!(package.source_type, "Release");
        assert_eq!(package.version, "0.5.1");
        assert!(package.has_update);
        assert_eq!(package.new_version.as_deref(), Some("0.5.2"));
        assert_eq!(package.repository_or_remote, "storytold/craft-launcher");
        assert!(package.description.contains("github"));
        match package.inner {
            UnifiedPackageSource::Release(release) => {
                assert_eq!(release.pkgname.as_deref(), Some("artcraft-launcher-bin"));
                assert_eq!(release.canal.as_deref(), Some("stable"));
            }
            other => panic!("expected the Release variant, got {other:?}"),
        }
    }

    #[test]
    fn alpm_and_aur_records_are_not_taken_from_the_daemon() {
        // The typed pacman list comes from the CLI backend; a second copy
        // from the snapshot would double every package, and AUR is a subset
        // of that same list.
        assert!(UnifiedPackage::from_snapshot(&record("ALPM", "ripgrep", "14.1.1")).is_none());
        assert!(UnifiedPackage::from_snapshot(&record("AUR", "yay", "12.4.2")).is_none());
        assert!(UnifiedPackage::from_snapshot(&record("Unknown", "x", "1")).is_none());
    }

    #[test]
    fn merging_keeps_the_typed_alpm_rows_and_appends_the_daemon_ecosystems() {
        let alpm = vec![AlpmPackage {
            name: "ripgrep".to_string(),
            version: "14.1.1".to_string(),
            ..Default::default()
        }];
        let snapshot = Snapshot {
            version: 1,
            generated_at: 0,
            packages: vec![
                record("ALPM", "ripgrep", "14.1.1"),
                record("AUR", "yay", "12.4.2"),
                record("Cargo", "cargo-edit", "0.13.11"),
                record("NPM", "typescript", "5.6.3"),
            ],
        };

        let merged = merge_snapshot(alpm, &snapshot);
        // One ALPM row from the backend, none from the snapshot, plus the
        // two ecosystems pacman does not own.
        assert_eq!(merged.len(), 3);
        assert_eq!(merged.iter().filter(|p| p.name == "ripgrep").count(), 1);

        let sources: Vec<&str> = merged.iter().map(|p| p.source_type.as_str()).collect();
        assert_eq!(sources, vec!["ALPM", "Cargo", "NPM"]);
        assert!(merged[0].is_installed);
    }

    #[test]
    fn merging_with_no_daemon_keeps_only_the_backend_packages() {
        let alpm = vec![AlpmPackage {
            name: "ripgrep".to_string(),
            version: "14.1.1".to_string(),
            ..Default::default()
        }];
        let merged = merge_snapshot(alpm, &Snapshot::default());
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source_type, "ALPM");
    }
}
