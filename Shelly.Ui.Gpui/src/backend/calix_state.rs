//! Reader for the unified ecosystem inventory maintained by the Calix engine.
//!
//! The Calix engine (`scripts/calix-sync.py`) keeps a single inventory of
//! every package ecosystem on the system under `$XDG_STATE_HOME/calix`.
//! The GPUI frontend reads that inventory to surface the ecosystems the
//! Shelly CLI backend does not know about: Cargo crates, globally installed
//! NPM packages, and packages tracked by the release backend.
//!
//! Every read is fault-tolerant: a missing or malformed state file degrades
//! to an empty section instead of failing the whole load, so a partially
//! synchronised system still renders.
//!
//! # Transitional
//!
//! This module reads files directly, which the Calix architecture forbids
//! for the final shape: the daemon is meant to be the single entry point,
//! serving a versioned normalised snapshot over a socket. This reader is the
//! temporary transport, and it disappears when the daemon serves the
//! inventory. The Python engine that produces these files is frozen and
//! awaits absorption into the Zig backend.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::backend::models::{AlpmPackage, UnifiedPackage};

/// A cargo crate installed under `~/.cargo/bin`.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct CargoCrate {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub bins: Vec<String>,
}

/// A globally installed NPM package.
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

/// Root of the Calix state directory (`$XDG_STATE_HOME/calix`).
pub fn state_dir() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
            PathBuf::from(home).join(".local/state")
        })
        .join("calix")
}

/// The unified ecosystem inventory written by the Calix engine.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EcosystemInventory {
    pub cargo: BTreeMap<String, CargoCrate>,
    pub npm: BTreeMap<String, NpmPackage>,
    pub releases: BTreeMap<String, ReleasePackage>,
}

impl EcosystemInventory {
    /// Reads the inventory from the standard Calix state directory.
    pub fn load() -> Self {
        Self::load_from(&state_dir())
    }

    /// Reads the inventory from an explicit state directory.
    pub fn load_from(root: &Path) -> Self {
        Self {
            cargo: read_state(root.join("cargo.json")),
            npm: read_state(root.join("npm.json")),
            releases: read_state(root.join("releases.json")),
        }
    }

    /// Total number of packages across every ecosystem in the inventory.
    pub fn total(&self) -> usize {
        self.cargo.len() + self.npm.len() + self.releases.len()
    }

    /// Converts the inventory into the unified package model used by the UI.
    pub fn to_unified_packages(&self) -> Vec<UnifiedPackage> {
        let mut packages = Vec::with_capacity(self.total());

        for (name, crate_) in &self.cargo {
            packages.push(UnifiedPackage::from_cargo(name.clone(), crate_.clone()));
        }
        for (name, package) in &self.npm {
            packages.push(UnifiedPackage::from_npm(name.clone(), package.clone()));
        }
        for (name, release) in &self.releases {
            packages.push(UnifiedPackage::from_release(name.clone(), release.clone()));
        }

        packages
    }
}

/// Merges the packages reported by the Shelly CLI backend with the
/// ecosystems tracked by the Calix inventory.
///
/// Both call sites (the initial load and the explicit reload) must go
/// through this function: keeping the merge in one place is what stops the
/// two paths from drifting apart.
pub fn merge_installed(
    alpm: Vec<AlpmPackage>,
    inventory: &EcosystemInventory,
) -> Vec<UnifiedPackage> {
    let mut packages: Vec<UnifiedPackage> = alpm
        .into_iter()
        .map(|pkg| UnifiedPackage::from_alpm(pkg, true))
        .collect();
    packages.extend(inventory.to_unified_packages());
    packages
}

/// Reads one state file, degrading to the default value on any failure.
fn read_state<T>(path: PathBuf) -> T
where
    T: for<'de> Deserialize<'de> + Default,
{
    match std::fs::read_to_string(&path) {
        Ok(raw) => match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(err) => {
                log::warn!("Calix state {} is malformed: {}", path.display(), err);
                T::default()
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => T::default(),
        Err(err) => {
            log::warn!("Calix state {} is unreadable: {}", path.display(), err);
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_state_dir(tag: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "calix-state-test-{}-{}",
            tag,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("create temp state dir");
        base
    }

    #[test]
    fn missing_state_directory_yields_empty_inventory() {
        let inventory = EcosystemInventory::load_from(Path::new("/nonexistent/calix/state"));
        assert_eq!(inventory.total(), 0);
        assert!(inventory.cargo.is_empty());
        assert!(inventory.npm.is_empty());
        assert!(inventory.releases.is_empty());
    }

    #[test]
    fn reads_cargo_npm_and_release_sections() {
        let root = temp_state_dir("sections");
        fs::write(
            root.join("cargo.json"),
            r#"{"ripgrep":{"version":"14.1.1","source":"crates.io","bins":["rg"]}}"#,
        )
        .unwrap();
        fs::write(
            root.join("npm.json"),
            r#"{"typescript":{"version":"5.6.3","overridden":false}}"#,
        )
        .unwrap();
        fs::write(
            root.join("releases.json"),
            r#"{"artcraft-launcher":{"installed_version":"0.5.1","latest_version":"0.5.2","update_available":true,"repo":"storytold/craft-launcher","provider":"github","canal":"stable"}}"#,
        )
        .unwrap();

        let inventory = EcosystemInventory::load_from(&root);
        assert_eq!(inventory.total(), 3);
        assert_eq!(inventory.cargo["ripgrep"].version, "14.1.1");
        assert_eq!(inventory.cargo["ripgrep"].bins, vec!["rg".to_string()]);
        assert_eq!(inventory.npm["typescript"].version, "5.6.3");
        assert!(inventory.releases["artcraft-launcher"].update_available);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn malformed_state_file_degrades_to_empty_section() {
        let root = temp_state_dir("malformed");
        fs::write(root.join("cargo.json"), "{ this is not json").unwrap();
        fs::write(
            root.join("npm.json"),
            r#"{"typescript":{"version":"5.6.3"}}"#,
        )
        .unwrap();

        let inventory = EcosystemInventory::load_from(&root);
        assert!(inventory.cargo.is_empty());
        assert_eq!(inventory.npm.len(), 1);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn cargo_crate_becomes_a_unified_package() {
        let crate_ = CargoCrate {
            version: "14.1.1".to_string(),
            source: "crates.io".to_string(),
            bins: vec!["rg".to_string()],
        };
        let package = UnifiedPackage::from_cargo("ripgrep".to_string(), crate_);
        assert_eq!(package.name, "ripgrep");
        assert_eq!(package.version, "14.1.1");
        assert_eq!(package.source_type, "Cargo");
        assert!(package.is_installed);
        assert!(!package.has_update);
    }

    #[test]
    fn release_with_older_installed_version_is_flagged_as_updatable() {
        let release = ReleasePackage {
            installed_version: Some("0.5.1".to_string()),
            latest_version: Some("0.5.2".to_string()),
            update_available: true,
            repo: Some("storytold/craft-launcher".to_string()),
            provider: Some("github".to_string()),
            canal: Some("stable".to_string()),
            pkgname: Some("artcraft-launcher-bin".to_string()),
        };
        let package = UnifiedPackage::from_release("artcraft-launcher".to_string(), release);
        assert_eq!(package.version, "0.5.1");
        assert!(package.has_update);
        assert_eq!(package.new_version.as_deref(), Some("0.5.2"));
        assert_eq!(package.source_type, "Release");
    }

    #[test]
    fn release_without_known_version_is_not_updatable() {
        let release = ReleasePackage {
            installed_version: None,
            latest_version: None,
            update_available: true,
            ..Default::default()
        };
        let package = UnifiedPackage::from_release("mystery".to_string(), release);
        assert!(!package.has_update);
        assert_eq!(package.version, "unknown");
    }

    #[test]
    fn merge_installed_combines_backend_and_ecosystem_packages() {
        let alpm = vec![AlpmPackage {
            name: "ripgrep".to_string(),
            version: "14.1.1".to_string(),
            ..Default::default()
        }];
        let mut inventory = EcosystemInventory::default();
        inventory.cargo.insert(
            "cargo-edit".to_string(),
            CargoCrate {
                version: "0.13.11".to_string(),
                source: "crates.io".to_string(),
                bins: vec!["cargo-add".to_string()],
            },
        );
        inventory.npm.insert(
            "typescript".to_string(),
            NpmPackage {
                version: "5.6.3".to_string(),
                overridden: false,
            },
        );
        inventory.releases.insert(
            "artcraft-launcher".to_string(),
            ReleasePackage {
                installed_version: Some("0.5.1".to_string()),
                latest_version: Some("0.5.1".to_string()),
                update_available: false,
                repo: Some("storytold/craft-launcher".to_string()),
                provider: Some("github".to_string()),
                canal: Some("stable".to_string()),
                pkgname: Some("artcraft-launcher-bin".to_string()),
            },
        );

        let merged = merge_installed(alpm, &inventory);
        assert_eq!(merged.len(), 4);

        let sources: Vec<&str> = merged.iter().map(|p| p.source_type.as_str()).collect();
        assert!(sources.contains(&"ALPM"));
        assert!(sources.contains(&"Cargo"));
        assert!(sources.contains(&"NPM"));
        assert!(sources.contains(&"Release"));

        // Le paquet ALPM doit conserver son statut installé.
        let alpm_pkg = merged
            .iter()
            .find(|p| p.source_type == "ALPM")
            .expect("ALPM package present");
        assert!(alpm_pkg.is_installed);
        assert_eq!(alpm_pkg.version, "14.1.1");
    }

    #[test]
    fn merge_installed_with_empty_inventory_keeps_only_backend_packages() {
        let alpm = vec![AlpmPackage {
            name: "ripgrep".to_string(),
            version: "14.1.1".to_string(),
            ..Default::default()
        }];
        let merged = merge_installed(alpm, &EcosystemInventory::default());
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source_type, "ALPM");
    }

    #[test]
    fn npm_package_carries_the_overridden_flag_in_its_description() {
        let package = UnifiedPackage::from_npm(
            "typescript".to_string(),
            NpmPackage {
                version: "5.6.3".to_string(),
                overridden: true,
            },
        );
        assert_eq!(package.source_type, "NPM");
        assert!(package.description.contains("overridden"));
    }
}
