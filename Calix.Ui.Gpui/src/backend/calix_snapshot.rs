//! The inventory document the Calix daemon serves.
//!
//! The daemon normalises every ecosystem into one flat record. The field
//! names here are the vocabulary the frontend already uses — `source_type`
//! and `repository_or_remote` — so a snapshot can be consumed without a
//! translation layer.
//!
//! The shape is flat rather than a tagged union on purpose: a union would
//! make every new ecosystem a breaking change for the consumer, whereas a
//! flat record lets the daemon add detail nobody reads and lets the
//! frontend ignore what it does not know.
//!
//! The contract is pinned on both sides: the Zig backend asserts the exact
//! serialised shape, and the fixture below was captured from it.

use serde::Deserialize;

/// The version of the snapshot format this frontend understands.
///
/// A snapshot with another version is refused rather than rendered
/// half-understood.
pub const SUPPORTED_SNAPSHOT_VERSION: u32 = 1;

/// A package as the daemon serves it.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct SnapshotPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    /// The ecosystem the package came from: `ALPM`, `AUR`, `Flatpak`,
    /// `AppImage`, `Cargo`, `NPM` or `Release`.
    pub source_type: String,
    #[serde(default)]
    pub repository_or_remote: String,
    #[serde(default = "default_installed")]
    pub is_installed: bool,
    #[serde(default)]
    pub has_update: bool,
    #[serde(default)]
    pub new_version: Option<String>,

    // ---- Optional detail, per ecosystem ------------------------------------
    #[serde(default)]
    pub size_on_disk: Option<u64>,
    #[serde(default)]
    pub icon_name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub update_url: Option<String>,
    #[serde(default)]
    pub repo_owner: Option<String>,
    #[serde(default)]
    pub repo_name: Option<String>,
    #[serde(default)]
    pub allow_prerelease: bool,
    #[serde(default)]
    pub overridden: bool,
    #[serde(default)]
    pub binaries: Vec<String>,
    /// The forge a release-managed package is built from, e.g. `github`.
    #[serde(default)]
    pub provider: Option<String>,
    /// The release channel a package follows, e.g. `stable`.
    #[serde(default)]
    pub canal: Option<String>,
    /// The package a release is built into.
    #[serde(default)]
    pub pkgname: Option<String>,
}

/// The inventory document.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Snapshot {
    /// The format version.
    pub version: u32,
    /// When the snapshot was produced, in Unix seconds.
    #[serde(default)]
    pub generated_at: i64,
    /// The normalised packages.
    #[serde(default)]
    pub packages: Vec<SnapshotPackage>,
}

impl Snapshot {
    /// Validates a document the daemon sent, refusing a format this
    /// frontend does not know.
    ///
    /// This is the only path a snapshot enters by, so it is also the only
    /// place the version is checked.
    pub fn from_value(value: serde_json::Value) -> Result<Self, String> {
        let snapshot: Snapshot =
            serde_json::from_value(value).map_err(|err| format!("invalid inventory: {err}"))?;
        if snapshot.version != SUPPORTED_SNAPSHOT_VERSION {
            return Err(format!(
                "unsupported inventory version {} (this frontend understands {})",
                snapshot.version, SUPPORTED_SNAPSHOT_VERSION
            ));
        }
        Ok(snapshot)
    }
}

fn default_installed() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/calix-inventory-sample.json");

    fn parse(raw: &str) -> Result<Snapshot, String> {
        let value: serde_json::Value =
            serde_json::from_str(raw).map_err(|err| format!("invalid inventory: {err}"))?;
        Snapshot::from_value(value)
    }

    fn of_source<'a>(snapshot: &'a Snapshot, source_type: &str) -> Vec<&'a SnapshotPackage> {
        snapshot
            .packages
            .iter()
            .filter(|package| package.source_type.eq_ignore_ascii_case(source_type))
            .collect()
    }

    #[test]
    fn parses_a_snapshot_captured_from_the_backend() {
        let snapshot = parse(FIXTURE).expect("the fixture must parse");
        assert_eq!(snapshot.version, SUPPORTED_SNAPSHOT_VERSION);
        assert!(!snapshot.packages.is_empty());
    }

    #[test]
    fn reads_the_optional_detail_of_every_ecosystem() {
        let snapshot = parse(FIXTURE).expect("the fixture must parse");

        let cargo = of_source(&snapshot, "Cargo");
        assert!(!cargo.is_empty(), "the fixture must carry Cargo packages");
        let crate_ = cargo[0];
        assert!(!crate_.version.is_empty());
        assert!(!crate_.repository_or_remote.is_empty());

        let npm = of_source(&snapshot, "NPM");
        assert!(!npm.is_empty(), "the fixture must carry NPM packages");

        let appimage = of_source(&snapshot, "AppImage");
        assert!(!appimage.is_empty(), "the fixture must carry AppImages");

        let releases = of_source(&snapshot, "Release");
        assert!(!releases.is_empty(), "the fixture must carry a Release package");
        assert_eq!(releases[0].provider.as_deref(), Some("github"));
        assert_eq!(releases[0].canal.as_deref(), Some("stable"));
        assert_eq!(
            releases[0].pkgname.as_deref(),
            Some("artcraft-launcher-bin")
        );
    }

    #[test]
    fn a_snapshot_of_an_unknown_version_is_refused() {
        let raw = r#"{"version":99,"packages":[]}"#;
        let error = parse(raw).expect_err("an unknown version must be refused");
        assert!(error.contains("unsupported inventory version"));
    }

    #[test]
    fn a_malformed_snapshot_is_refused() {
        assert!(parse("{not json").is_err());
    }

    #[test]
    fn an_absent_optional_field_is_not_an_error() {
        let raw = r#"{"version":1,"packages":[{"name":"x","version":"1","source_type":"Cargo"}]}"#;
        let snapshot = parse(raw).expect("a sparse record must parse");
        let package = &snapshot.packages[0];
        assert_eq!(package.name, "x");
        assert!(package.is_installed, "installed defaults to true");
        assert!(package.description.is_empty());
        assert!(package.binaries.is_empty());
    }
}
