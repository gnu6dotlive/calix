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

use crate::backend::calix_state::{CargoCrate, NpmPackage, ReleasePackage};

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
