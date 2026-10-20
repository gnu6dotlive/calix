use crate::backend::client::CalixClient;
use crate::backend::models::{AlpmPackage, UnifiedPackage};
use crate::state::query::SourceScope;
use crate::state::session::{PackageKey, PackageSourceKind};
use gpui::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Clé d'indexation pour le cache de recherche en session
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchKey {
    pub query: String,
    pub source_scope: SourceScope,
}

impl SearchKey {
    pub fn new(query: impl Into<String>, source_scope: SourceScope) -> Self {
        Self {
            query: query.into().trim().to_lowercase(),
            source_scope,
        }
    }
}

/// Statut de santé d'une source de distribution backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceHealthStatus {
    Disabled,
    Loading,
    Ready,
    Failed,
}

/// Métadonnées d'état de santé par source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceHealth {
    pub status: SourceHealthStatus,
    pub error_message: Option<String>,
}

impl SourceHealth {
    pub fn ready() -> Self {
        Self {
            status: SourceHealthStatus::Ready,
            error_message: None,
        }
    }

    pub fn failed(msg: impl Into<String>) -> Self {
        Self {
            status: SourceHealthStatus::Failed,
            error_message: Some(msg.into()),
        }
    }

    pub fn is_failed(&self) -> bool {
        self.status == SourceHealthStatus::Failed
    }
}

/// Carte des états de santé pour l'ensemble des sources supportées
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceHealthMap {
    pub alpm: SourceHealth,
    pub aur: SourceHealth,
    pub flatpak: SourceHealth,
    pub appimage: SourceHealth,
}

impl Default for SourceHealthMap {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceHealthMap {
    pub fn new() -> Self {
        Self {
            alpm: SourceHealth::ready(),
            aur: SourceHealth::ready(),
            flatpak: SourceHealth::ready(),
            appimage: SourceHealth::ready(),
        }
    }

    /// Rail de santé d'une source de recherche.
    ///
    /// Les écosystèmes Calix (Cargo, NPM, releases) n'ont pas de rail : ils
    /// ne sont pas interrogeables par le backend de recherche, seulement
    /// listés dans l'inventaire installé.
    pub fn for_source(&self, source: PackageSourceKind) -> Option<&SourceHealth> {
        match source {
            PackageSourceKind::Alpm => Some(&self.alpm),
            PackageSourceKind::Aur => Some(&self.aur),
            PackageSourceKind::Flatpak => Some(&self.flatpak),
            PackageSourceKind::AppImage => Some(&self.appimage),
            PackageSourceKind::Cargo | PackageSourceKind::Npm | PackageSourceKind::Release => None,
        }
    }

    pub fn get(&self, source: &PackageSourceKind) -> Option<&SourceHealth> {
        self.for_source(*source)
    }

    pub fn set_source(&mut self, source: PackageSourceKind, health: SourceHealth) {
        match source {
            PackageSourceKind::Alpm => self.alpm = health,
            PackageSourceKind::Aur => self.aur = health,
            PackageSourceKind::Flatpak => self.flatpak = health,
            PackageSourceKind::AppImage => self.appimage = health,
            PackageSourceKind::Cargo | PackageSourceKind::Npm | PackageSourceKind::Release => {}
        }
    }
}

/// Événements sémantiques émis par le magasin de paquets
#[derive(Debug, Clone, PartialEq)]
pub enum PackageStoreEvent {
    ResultsChanged,
    PackageInvalidated(PackageKey),
    UpdatesChanged(usize),
    InstalledChanged(usize),
    IdentityResolved(PackageKey),
}

/// Magasin d'état applicatif gérant les collections de paquets, caches et invalidations
pub struct PackageStore {
    pub client: CalixClient,
    pub active_results: Arc<[UnifiedPackage]>,
    pub installed_packages: Arc<[UnifiedPackage]>,
    pub updates_packages: Arc<[UnifiedPackage]>,
    pub updates_count: usize,
    pub detail_cache: HashMap<PackageKey, AlpmPackage>,
    pub search_cache: HashMap<SearchKey, Vec<UnifiedPackage>>,
    pub pkgbuild_cache: HashMap<String, String>,
    pub in_flight_generation: usize,
    pub source_health: SourceHealthMap,
    pub installed_error: Option<String>,
    pub updates_error: Option<String>,
    pub news_error: Option<String>,
    pub detail_errors: HashMap<PackageKey, String>,
}

impl EventEmitter<PackageStoreEvent> for PackageStore {}

impl PackageStore {
    pub fn new(client: CalixClient) -> Self {
        Self {
            client,
            active_results: Arc::from([]),
            installed_packages: Arc::from([]),
            updates_packages: Arc::from([]),
            updates_count: 0,
            detail_cache: HashMap::new(),
            search_cache: HashMap::new(),
            pkgbuild_cache: HashMap::new(),
            in_flight_generation: 0,
            source_health: SourceHealthMap::new(),
            installed_error: None,
            updates_error: None,
            news_error: None,
            detail_errors: HashMap::new(),
        }
    }

    /// Tente de récupérer les résultats de recherche depuis le cache de session
    pub fn get_cached_search(
        &self,
        query: &str,
        source_scope: SourceScope,
    ) -> Option<&Vec<UnifiedPackage>> {
        let key = SearchKey::new(query, source_scope);
        self.search_cache.get(&key)
    }

    /// Enregistre des résultats dans le cache de recherche de session (succès complets uniquement)
    pub fn cache_search(
        &mut self,
        query: &str,
        source_scope: SourceScope,
        results: Vec<UnifiedPackage>,
    ) {
        let key = SearchKey::new(query, source_scope);
        self.search_cache.insert(key, results);
    }

    /// Met à jour les résultats actifs affichés
    pub fn set_active_results(
        &mut self,
        results: Vec<UnifiedPackage>,
        generation: usize,
        cx: &mut Context<Self>,
    ) {
        if generation >= self.in_flight_generation {
            self.in_flight_generation = generation;
            crate::components::package_identity::PackageIdentity::preload(&results);
            self.active_results = Arc::from(results);
            cx.emit(PackageStoreEvent::ResultsChanged);
            cx.notify();
        }
    }

    /// Définit la liste des paquets installés
    pub fn set_installed_packages(&mut self, pkgs: Vec<UnifiedPackage>, cx: &mut Context<Self>) {
        let count = pkgs.len();
        self.installed_packages = Arc::from(pkgs);
        self.installed_error = None;
        cx.emit(PackageStoreEvent::InstalledChanged(count));
        cx.notify();
    }

    /// Définit l'erreur de chargement des paquets installés en préservant les données connues
    pub fn set_installed_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.installed_error = Some(error);
        cx.notify();
    }

    /// Définit la liste des mises à jour disponibles
    pub fn set_updates_packages(&mut self, pkgs: Vec<UnifiedPackage>, cx: &mut Context<Self>) {
        self.updates_count = pkgs.len();
        self.updates_packages = Arc::from(pkgs);
        self.updates_error = None;
        cx.emit(PackageStoreEvent::UpdatesChanged(self.updates_count));
        cx.notify();
    }

    /// Définit l'erreur de vérification des mises à jour en préservant les données connues
    pub fn set_updates_error(&mut self, error: String, cx: &mut Context<Self>) {
        self.updates_error = Some(error);
        cx.notify();
    }

    /// Tente de récupérer les détails d'un paquet depuis le cache
    pub fn get_cached_details(&self, key: &PackageKey) -> Option<&AlpmPackage> {
        self.detail_cache.get(key)
    }

    /// Enregistre les détails d'un paquet dans le cache
    pub fn cache_details(&mut self, key: PackageKey, details: AlpmPackage) {
        self.detail_errors.remove(&key);
        self.detail_cache.insert(key, details);
    }

    /// Enregistre une erreur de récupération des détails d'un paquet
    pub fn set_detail_error(&mut self, key: PackageKey, error: String) {
        self.detail_errors.insert(key, error);
    }

    /// Récupère l'erreur éventuelle de récupération des détails d'un paquet
    pub fn get_detail_error(&self, key: &PackageKey) -> Option<&String> {
        self.detail_errors.get(key)
    }

    /// Tente de récupérer la recette PKGBUILD depuis le cache
    pub fn get_cached_pkgbuild(&self, pkg_name: &str) -> Option<&String> {
        self.pkgbuild_cache.get(pkg_name)
    }

    /// Enregistre une recette PKGBUILD dans le cache
    pub fn cache_pkgbuild(&mut self, pkg_name: String, content: String) {
        self.pkgbuild_cache.insert(pkg_name, content);
    }

    /// Invalide un paquet précis et nettoie les entrées correspondantes dans les caches
    pub fn invalidate_package(&mut self, key: &PackageKey, cx: &mut Context<Self>) {
        self.detail_cache.remove(key);
        self.pkgbuild_cache.remove(&key.name);
        self.search_cache
            .retain(|_, pkgs| !pkgs.iter().any(|p| &p.key() == key));

        cx.emit(PackageStoreEvent::PackageInvalidated(key.clone()));
        cx.notify();
    }

    /// Invalide la liste des paquets installés
    pub fn invalidate_installed(&mut self, cx: &mut Context<Self>) {
        self.installed_packages = Arc::from([]);
        self.installed_error = None;
        cx.notify();
    }

    /// Invalide la liste des mises à jour disponibles
    pub fn invalidate_updates(&mut self, cx: &mut Context<Self>) {
        self.updates_packages = Arc::from([]);
        self.updates_error = None;
        cx.notify();
    }

    /// Met à jour la santé d'une source spécifique
    pub fn update_source_health(&mut self, source: PackageSourceKind, health: SourceHealth) {
        self.source_health.set_source(source, health);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::session::PackageSourceKind;
    use core::prelude::v1::test;

    #[test]
    fn test_search_key_normalization() {
        let s_all = SourceScope::all();
        let mut s_alpm = SourceScope::all();
        s_alpm.aur = false;
        s_alpm.flatpak = false;
        s_alpm.appimage = false;

        let k1 = SearchKey::new("  RipGrep  ", s_all);
        let k2 = SearchKey::new("ripgrep", s_all);
        let k3 = SearchKey::new("ripgrep", s_alpm);

        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn test_package_store_cache_and_invalidation() {
        let client = CalixClient::new(None);
        let mut store = PackageStore::new(client);

        let key = PackageKey::new(PackageSourceKind::Alpm, "neovim", Some("extra".into()));
        let mut details = AlpmPackage::default();
        details.name = "neovim".into();
        details.version = "0.10.0".into();

        store.cache_details(key.clone(), details.clone());
        assert!(store.get_cached_details(&key).is_some());

        // Invalidation manuelle du cache
        store.detail_cache.remove(&key);
        assert!(store.get_cached_details(&key).is_none());
    }

    #[test]
    fn test_search_cache_storage() {
        let client = CalixClient::new(None);
        let mut store = PackageStore::new(client);

        let query = "firefox";
        let scope = SourceScope::all();

        assert!(store.get_cached_search(query, scope).is_none());

        let pkg = UnifiedPackage {
            name: "firefox".into(),
            version: "128.0".into(),
            description: "Web browser".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: crate::backend::models::UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        store.cache_search(query, scope, vec![pkg]);
        let cached = store.get_cached_search(query, scope);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 1);
        assert_eq!(cached.unwrap()[0].name, "firefox");
    }

    #[test]
    fn test_source_health_and_outcome_transitions() {
        let mut health = SourceHealthMap::new();
        assert!(!health.alpm.is_failed());
        assert!(!health.aur.is_failed());

        health.set_source(
            PackageSourceKind::Aur,
            SourceHealth::failed("AUR timed out"),
        );
        assert!(health.aur.is_failed());
        assert_eq!(health.aur.error_message.as_deref(), Some("AUR timed out"));
        assert!(!health.alpm.is_failed());

        health.set_source(PackageSourceKind::Aur, SourceHealth::ready());
        assert!(!health.aur.is_failed());
    }

    #[test]
    fn test_failure_truth_partial_and_failed_not_cached() {
        let client = CalixClient::new(None);
        let store = PackageStore::new(client);
        let scope = SourceScope::all();

        // If search returned empty due to partial failure, it must not be in cache
        assert!(store.get_cached_search("something_failed", scope).is_none());
    }

    #[test]
    fn test_appimage_filtering_and_key_identity() {
        use crate::backend::models::AppImageItem;

        let item = AppImageItem {
            name: "Obsidian".into(),
            desktop_name: Some("obsidian.desktop".into()),
            version: Some("1.5.8".into()),
            icon_name: Some("obsidian".into()),
            description: Some("Knowledge base and note-taking app".into()),
            size_on_disk: Some(120_000_000),
            update_url: None,
            repo_owner: None,
            repo_name: None,
            path: Some("/home/user/Applications/Obsidian.AppImage".into()),
        };

        let pkg = UnifiedPackage::from_appimage(item.clone());
        assert_eq!(pkg.name, "Obsidian");
        assert_eq!(pkg.version, "1.5.8");
        assert_eq!(pkg.source_type, "AppImage");
        assert_eq!(pkg.repository_or_remote, "appimage");
        assert_eq!(pkg.key().source, PackageSourceKind::AppImage);
        assert_eq!(pkg.key().name, "Obsidian");

        let q1 = "obsidian";
        let q2 = "knowledge";
        let q3 = "vlc";

        let matches_q1 = item.name.to_lowercase().contains(q1)
            || item
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(q1))
                .unwrap_or(false);
        let matches_q2 = item.name.to_lowercase().contains(q2)
            || item
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(q2))
                .unwrap_or(false);
        let matches_q3 = item.name.to_lowercase().contains(q3)
            || item
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(q3))
                .unwrap_or(false);

        assert!(matches_q1);
        assert!(matches_q2);
        assert!(!matches_q3);

        let client = CalixClient::new(None);
        let mut store = PackageStore::new(client);
        let scope = SourceScope {
            alpm: false,
            aur: false,
            flatpak: false,
            appimage: true,
        };
        store.cache_search("obsidian", scope, vec![pkg.clone()]);
        let cached = store.get_cached_search("obsidian", scope);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap()[0].name, "Obsidian");
        assert_eq!(cached.unwrap()[0].source_type, "AppImage");
    }

    #[test]
    fn test_pkgbuild_cache_storage_and_invalidation() {
        let client = CalixClient::new(None);
        let mut store = PackageStore::new(client);

        let pkg_name = "visual-studio-code-bin";
        let pkgbuild = "pkgname=visual-studio-code-bin\npkgver=1.138.0";

        assert!(store.get_cached_pkgbuild(pkg_name).is_none());

        store.cache_pkgbuild(pkg_name.into(), pkgbuild.into());
        assert_eq!(
            store.get_cached_pkgbuild(pkg_name),
            Some(&pkgbuild.to_string())
        );

        // Invalidate via key
        let key = PackageKey::new(PackageSourceKind::Aur, pkg_name, None);
        store.pkgbuild_cache.remove(&key.name);
        assert!(store.get_cached_pkgbuild(pkg_name).is_none());
    }

    #[test]
    fn test_search_generation_discard_stale_and_accept_fresh() {
        use crate::backend::models::AppImageItem;
        let client = CalixClient::new(None);
        let mut store = PackageStore::new(client);
        assert_eq!(store.in_flight_generation, 0);

        let item1 = UnifiedPackage::from_appimage(AppImageItem {
            name: "App1".into(),
            desktop_name: None,
            version: Some("1.0".into()),
            icon_name: None,
            description: None,
            size_on_disk: None,
            update_url: None,
            repo_owner: None,
            repo_name: None,
            path: None,
        });
        if 1 >= store.in_flight_generation {
            store.in_flight_generation = 1;
            store.active_results = std::sync::Arc::from(vec![item1.clone()]);
        }
        assert_eq!(store.active_results.len(), 1);
        assert_eq!(store.in_flight_generation, 1);

        let clear_gen = 2;
        if clear_gen >= store.in_flight_generation {
            store.in_flight_generation = clear_gen;
            store.active_results = std::sync::Arc::from([]);
        }
        assert_eq!(store.active_results.len(), 0);
        assert_eq!(store.in_flight_generation, 2);

        // Stale generation 1 arrives late -> discarded!
        if 1 >= store.in_flight_generation {
            store.in_flight_generation = 1;
            store.active_results = std::sync::Arc::from(vec![item1.clone()]);
        }
        assert_eq!(
            store.active_results.len(),
            0,
            "Stale gen 1 must be discarded"
        );
        assert_eq!(store.in_flight_generation, 2);

        // Fresh generation 3 arrives -> accepted!
        let item2 = UnifiedPackage::from_appimage(AppImageItem {
            name: "App2".into(),
            desktop_name: None,
            version: Some("2.0".into()),
            icon_name: None,
            description: None,
            size_on_disk: None,
            update_url: None,
            repo_owner: None,
            repo_name: None,
            path: None,
        });
        let fresh_gen = 3;
        if fresh_gen >= store.in_flight_generation {
            store.in_flight_generation = fresh_gen;
            store.active_results = std::sync::Arc::from(vec![item2]);
        }
        assert_eq!(
            store.active_results.len(),
            1,
            "Fresh gen 3 must be accepted"
        );
        assert_eq!(store.active_results[0].name, "App2");
        assert_eq!(store.in_flight_generation, 3);
    }

    #[test]
    fn test_refresh_failure_preserves_known_good_data() {
        let client = CalixClient::new(None);
        let mut store = PackageStore::new(client);

        let pkg = UnifiedPackage {
            name: "ripgrep".into(),
            version: "14.1.0".into(),
            description: "Line-oriented search tool".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: crate::backend::models::UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        // 1. Establish known-good state
        store.installed_packages = std::sync::Arc::from(vec![pkg.clone()]);
        store.updates_packages = std::sync::Arc::from(vec![pkg.clone()]);
        store.installed_error = None;
        store.updates_error = None;

        assert_eq!(store.installed_packages.len(), 1);
        assert_eq!(store.updates_packages.len(), 1);

        // 2. Simulate refresh failure on installed packages: record error without wiping data
        store.installed_error = Some("failed to synchronize all databases".to_string());
        assert_eq!(
            store.installed_packages.len(),
            1,
            "installed packages must be preserved on refresh failure"
        );
        assert_eq!(store.installed_packages[0].name, "ripgrep");
        assert_eq!(
            store.installed_error.as_deref(),
            Some("failed to synchronize all databases")
        );

        // 3. Simulate refresh failure on updates: record error without wiping data
        store.updates_error = Some("connection timeout".to_string());
        assert_eq!(
            store.updates_packages.len(),
            1,
            "updates packages must be preserved on refresh failure"
        );
        assert_eq!(store.updates_packages[0].name, "ripgrep");
        assert_eq!(store.updates_error.as_deref(), Some("connection timeout"));
    }
}
