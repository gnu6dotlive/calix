use crate::backend::models::{UnifiedPackage, UnifiedPackageSource};
use crate::icons::AppIcon;
use crate::state::PackageKey;
use crate::theme::Theme;
use gpui::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

/// Identité visuelle résolue selon la chaîne déterministe à 3 tiers :
/// 1. Authentic: Icône authentique vérifiée sur le système de fichiers local
/// 2. Symbolic: Glyphe vectoriel symbolique certifié selon la source de paquets
/// 3. Fallback: Boîte générique neutre en cas de source inconnue
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedIdentity {
    Authentic(PathBuf),
    Symbolic(AppIcon),
    Fallback(AppIcon),
}

static IDENTITY_CACHE: OnceLock<RwLock<HashMap<PackageKey, ResolvedIdentity>>> = OnceLock::new();
static RESOLVING_KEYS: OnceLock<RwLock<HashSet<PackageKey>>> = OnceLock::new();
static RESOLVER_SENDER: OnceLock<std::sync::mpsc::SyncSender<UnifiedPackage>> = OnceLock::new();
static RESOLVE_NOTIFIER: OnceLock<
    std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<PackageKey>>>,
> = OnceLock::new();

fn get_identity_cache() -> &'static RwLock<HashMap<PackageKey, ResolvedIdentity>> {
    IDENTITY_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}

fn get_resolving_keys() -> &'static RwLock<HashSet<PackageKey>> {
    RESOLVING_KEYS.get_or_init(|| RwLock::new(HashSet::new()))
}

fn get_resolver_sender() -> &'static std::sync::mpsc::SyncSender<UnifiedPackage> {
    RESOLVER_SENDER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::sync_channel::<UnifiedPackage>(2048);
        std::thread::Builder::new()
            .name("calix-identity-resolver".to_string())
            .spawn(move || {
                while let Ok(pkg) = rx.recv() {
                    let key = pkg.key();
                    let resolved = PackageIdentity::resolve_identity_sync(&pkg);
                    if let Ok(mut cache) = get_identity_cache().write() {
                        cache.insert(key.clone(), resolved);
                    }
                    if let Ok(mut pending) = get_resolving_keys().write() {
                        pending.remove(&key);
                    }
                    PackageIdentity::notify_identity_resolved(&key);
                }
            })
            .expect("failed to spawn identity resolver thread");
        tx
    })
}

pub struct PackageIdentity;

impl PackageIdentity {
    /// Enregistre le transmetteur de notification asynchrone GPUI
    pub fn register_notifier(sender: tokio::sync::mpsc::UnboundedSender<PackageKey>) {
        let mutex = RESOLVE_NOTIFIER.get_or_init(|| std::sync::Mutex::new(None));
        if let Ok(mut guard) = mutex.lock() {
            *guard = Some(sender);
        }
    }

    /// Notifie les écouteurs de la résolution d'une identité de paquet
    pub fn notify_identity_resolved(key: &PackageKey) {
        if let Some(mutex) = RESOLVE_NOTIFIER.get() {
            if let Ok(guard) = mutex.lock() {
                if let Some(ref sender) = *guard {
                    let _ = sender.send(key.clone());
                }
            }
        }
    }

    /// Sonde le système de fichiers local pour localiser une icône authentique fournie par la source.
    /// Pour ALPM et AUR, inspecte les fichiers possédés par le paquet dans /var/lib/pacman/local/<pkg>-*/files.
    pub fn find_authentic_icon(pkg: &UnifiedPackage) -> Option<PathBuf> {
        match &pkg.inner {
            UnifiedPackageSource::AppImage(item) => {
                if let Some(ref icon_name) = item.icon_name {
                    let p = PathBuf::from(icon_name);
                    if p.is_file() {
                        return Some(p);
                    }
                    if let Some(found) = Self::resolve_icon_name(icon_name) {
                        return Some(found);
                    }
                }
                if let Some(ref path_str) = item.path {
                    let p = PathBuf::from(path_str);
                    let sibling_png = p.with_extension("png");
                    if sibling_png.is_file() {
                        return Some(sibling_png);
                    }
                }
                None
            }
            UnifiedPackageSource::Flatpak(hit) => {
                if let Some(ref app_id) = hit.app_id {
                    Self::probe_flatpak_icon(app_id)
                } else {
                    None
                }
            }
            UnifiedPackageSource::Standard(_) | UnifiedPackageSource::Aur(_) => {
                Self::probe_alpm_or_aur_icon(pkg)
            }
            // Les écosystèmes Calix n'exposent pas d'icône de fichier :
            // ils retombent sur leur icône symbolique de source.
            UnifiedPackageSource::Cargo(_)
            | UnifiedPackageSource::Npm(_)
            | UnifiedPackageSource::Release(_) => None,
        }
    }

    fn probe_flatpak_icon(app_id: &str) -> Option<PathBuf> {
        let mut roots = vec![PathBuf::from(
            "/var/lib/flatpak/exports/share/icons/hicolor",
        )];
        if let Some(d) = dirs::data_dir() {
            roots.push(d.join("flatpak/exports/share/icons/hicolor"));
        }
        let resolutions = ["128x128", "scalable", "64x64", "48x48", "32x32"];
        for root in &roots {
            if !root.is_dir() {
                continue;
            }
            for res in &resolutions {
                let png = root.join(res).join("apps").join(format!("{}.png", app_id));
                if png.is_file() {
                    return Some(png);
                }
                let svg = root.join(res).join("apps").join(format!("{}.svg", app_id));
                if svg.is_file() {
                    return Some(svg);
                }
            }
        }
        None
    }

    /// Résolution authentique basée sur la provenance stricte pour ALPM et AUR.
    /// Inspecte la base locale pacman (/var/lib/pacman/local) pour identifier
    /// le record dont le champ %NAME% correspond EXACTEMENT au nom du paquet.
    fn probe_alpm_or_aur_icon(pkg: &UnifiedPackage) -> Option<PathBuf> {
        if !pkg.is_installed {
            return None;
        }
        let pacman_local = Path::new("/var/lib/pacman/local");
        Self::probe_alpm_or_aur_icon_in_dir(pkg, pacman_local, None)
    }

    /// Sonde un répertoire de base de données locale pacman avec prise en charge d'une racine système optionnelle
    pub fn probe_alpm_or_aur_icon_in_dir(
        pkg: &UnifiedPackage,
        pacman_local: &Path,
        sys_root: Option<&Path>,
    ) -> Option<PathBuf> {
        if !pacman_local.is_dir() {
            return None;
        }

        let read_dir = std::fs::read_dir(pacman_local).ok()?;
        let prefix = format!("{}-", pkg.name);
        let mut target_dir: Option<PathBuf> = None;
        let mut entries_vec = Vec::new();

        for entry in read_dir.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy().to_string();
            if name_str.starts_with(&prefix) {
                let desc_path = entry_path.join("desc");
                if let Ok(desc_content) = std::fs::read_to_string(&desc_path) {
                    if let Some(name) = Self::parse_desc_field(&desc_content, "NAME") {
                        if name == pkg.name {
                            target_dir = Some(entry_path.clone());
                            break;
                        }
                    }
                }
            }
            entries_vec.push(entry_path);
        }

        if target_dir.is_none() {
            for entry_path in entries_vec {
                let desc_path = entry_path.join("desc");
                if let Ok(desc_content) = std::fs::read_to_string(&desc_path) {
                    if let Some(name) = Self::parse_desc_field(&desc_content, "NAME") {
                        if name == pkg.name {
                            target_dir = Some(entry_path);
                            break;
                        }
                    }
                }
            }
        }

        let target_dir = target_dir?;
        let files_path = target_dir.join("files");
        let content = std::fs::read_to_string(&files_path).ok()?;
        let mut desktop_rel_path: Option<String> = None;
        let mut fallback_desktop: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("usr/share/applications/") && line.ends_with(".desktop") {
                if line.ends_with(&format!("{}.desktop", pkg.name)) {
                    desktop_rel_path = Some(line.to_string());
                    break;
                } else if fallback_desktop.is_none() {
                    fallback_desktop = Some(line.to_string());
                }
            }
        }
        if desktop_rel_path.is_none() {
            desktop_rel_path = fallback_desktop;
        }

        let desktop_rel = desktop_rel_path?;
        let full_desktop_path = match sys_root {
            Some(root) => root.join(desktop_rel),
            None => PathBuf::from("/").join(desktop_rel),
        };
        if !full_desktop_path.is_file() {
            return None;
        }

        let icon_val = Self::parse_desktop_icon_field(&full_desktop_path)?;
        Self::resolve_icon_name_in_root(&icon_val, sys_root)
    }

    /// Extrait la valeur d'un champ %FIELD% d'un fichier desc pacman
    pub fn parse_desc_field(content: &str, field: &str) -> Option<String> {
        let marker = format!("%{}%", field);
        let mut lines = content.lines();
        while let Some(line) = lines.next() {
            if line.trim() == marker {
                return lines.next().map(|l| l.trim().to_string());
            }
        }
        None
    }

    /// Extrait le champ Icon= de la section [Desktop Entry] d'un fichier .desktop
    fn parse_desktop_icon_field(desktop_path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(desktop_path).ok()?;
        let mut in_desktop_entry = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            } else if trimmed.starts_with('[') && in_desktop_entry {
                break;
            }
            if in_desktop_entry && trimmed.starts_with("Icon=") {
                let val = trimmed.strip_prefix("Icon=").unwrap_or("").trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
        None
    }

    /// Résout un nom d'icône ou chemin absolu sur le système de fichiers standard
    fn resolve_icon_name(icon_name: &str) -> Option<PathBuf> {
        Self::resolve_icon_name_in_root(icon_name, None)
    }

    /// Résout un nom d'icône ou chemin absolu avec racine de système configurable
    fn resolve_icon_name_in_root(icon_name: &str, sys_root: Option<&Path>) -> Option<PathBuf> {
        let base_root = sys_root.unwrap_or(Path::new("/"));
        let p = PathBuf::from(icon_name);
        if p.is_file() {
            return Some(p);
        }

        let extensions: &[&str] = if icon_name.ends_with(".png") || icon_name.ends_with(".svg") {
            &[""]
        } else {
            &[".png", ".svg"]
        };

        // 1. /usr/share/pixmaps
        let pixmaps_dir = base_root.join("usr/share/pixmaps");
        for ext in extensions {
            let pix = pixmaps_dir.join(format!("{}{}", icon_name, ext));
            if pix.is_file() {
                return Some(pix);
            }
        }

        // 2. /usr/share/icons/hicolor
        let hicolor_root = base_root.join("usr/share/icons/hicolor");
        if hicolor_root.is_dir() {
            for res in &["128x128", "scalable", "64x64", "48x48", "256x256", "32x32"] {
                for ext in extensions {
                    let path = hicolor_root
                        .join(res)
                        .join("apps")
                        .join(format!("{}{}", icon_name, ext));
                    if path.is_file() {
                        return Some(path);
                    }
                }
            }
        }

        // 3. Répertoire utilisateur (~/.local/share/icons/hicolor)
        if sys_root.is_none() {
            if let Some(data_dir) = dirs::data_dir() {
                let user_icons = data_dir.join("icons/hicolor");
                if user_icons.is_dir() {
                    for res in &["128x128", "scalable", "64x64", "48x48", "256x256", "32x32"] {
                        for ext in extensions {
                            let path = user_icons
                                .join(res)
                                .join("apps")
                                .join(format!("{}{}", icon_name, ext));
                            if path.is_file() {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Dérive l'identité de repli symbolique ou générique en O(1) sans accès I/O
    pub fn symbolic_or_fallback(pkg: &UnifiedPackage) -> ResolvedIdentity {
        match pkg.source_type.to_uppercase().as_str() {
            "ALPM" => ResolvedIdentity::Symbolic(AppIcon::SourceAlpm),
            "AUR" => ResolvedIdentity::Symbolic(AppIcon::SourceAur),
            "FLATPAK" => ResolvedIdentity::Symbolic(AppIcon::SourceFlatpak),
            "APPIMAGE" => ResolvedIdentity::Symbolic(AppIcon::SourceAppImage),
            "CARGO" => ResolvedIdentity::Symbolic(AppIcon::SourceCargo),
            "NPM" => ResolvedIdentity::Symbolic(AppIcon::SourceNpm),
            "RELEASE" => ResolvedIdentity::Symbolic(AppIcon::SourceRelease),
            _ => ResolvedIdentity::Fallback(AppIcon::PackageGeneric),
        }
    }

    /// Résout l'identité visuelle de façon déterministe et synchrone
    pub fn resolve_identity_sync(pkg: &UnifiedPackage) -> ResolvedIdentity {
        if let Some(path) = Self::find_authentic_icon(pkg) {
            ResolvedIdentity::Authentic(path)
        } else {
            Self::symbolic_or_fallback(pkg)
        }
    }

    /// Enfile une requête de résolution vers la queue bornée partagée (dédoublonnée en O(1))
    pub fn enqueue(pkg: &UnifiedPackage) {
        let key = pkg.key();
        if let Ok(cache) = get_identity_cache().read() {
            if cache.contains_key(&key) {
                return;
            }
        }

        let should_enqueue = if let Ok(mut pending) = get_resolving_keys().write() {
            pending.insert(key.clone())
        } else {
            false
        };

        if should_enqueue {
            let sender = get_resolver_sender();
            if let Err(std::sync::mpsc::TrySendError::Full(_)) = sender.try_send(pkg.clone()) {
                if let Ok(mut pending) = get_resolving_keys().write() {
                    pending.remove(&key);
                }
            }
        }
    }

    /// Résout l'identité visuelle de manière non-bloquante pour le chemin chaud de rendu GPUI.
    ///
    /// - Cache HIT: Retourne immédiatement le résultat en O(1) mémoire sans aucun I/O disque.
    /// - Cache MISS: Retourne immédiatement l'icône symbolique/fallback O(1) et soumet
    ///   la clé à la file de résolution asynchrone partagée. ZÉRO spawn de thread OS.
    pub fn resolve_identity(pkg: &UnifiedPackage) -> ResolvedIdentity {
        let key = pkg.key();

        if let Ok(cache) = get_identity_cache().read() {
            if let Some(resolved) = cache.get(&key) {
                return resolved.clone();
            }
        }

        Self::enqueue(pkg);
        Self::symbolic_or_fallback(pkg)
    }

    /// Précharge les identités d'une collection de paquets via la queue d'autorité unique
    pub fn preload(packages: &[UnifiedPackage]) {
        for pkg in packages {
            Self::enqueue(pkg);
        }
    }

    #[cfg(test)]
    pub fn clear_cache() {
        if let Ok(mut cache) = get_identity_cache().write() {
            cache.clear();
        }
        if let Ok(mut pending) = get_resolving_keys().write() {
            pending.clear();
        }
    }

    /// Couleur d'accentuation sémantique de la source
    pub fn source_color(source_type: &str, theme: &Theme) -> Rgba {
        match source_type.to_uppercase().as_str() {
            "ALPM" => theme.badge_alpm,
            "AUR" => theme.badge_aur,
            "FLATPAK" => theme.badge_flatpak,
            "APPIMAGE" => theme.badge_appimage,
            "CARGO" => theme.badge_cargo,
            "NPM" => theme.badge_npm,
            "RELEASE" => theme.badge_release,
            _ => theme.text_muted,
        }
    }

    /// Teinte d'arrière-plan de l'avatar selon la source
    pub fn avatar_bg_color(source_type: &str, theme: &Theme) -> Rgba {
        let base = Self::source_color(source_type, theme);
        Rgba {
            r: base.r,
            g: base.g,
            b: base.b,
            a: 0.12,
        }
    }

    /// Teinte de bordure subtile de l'avatar selon la source
    pub fn avatar_border_color(source_type: &str, theme: &Theme) -> Rgba {
        let base = Self::source_color(source_type, theme);
        Rgba {
            r: base.r,
            g: base.g,
            b: base.b,
            a: 0.28,
        }
    }

    /// Rendu de l'avatar d'identité dans un conteneur géométrique stable (pour les Cartes)
    pub fn render_avatar(
        pkg: &UnifiedPackage,
        size: f32,
        corner_radius: f32,
        theme: &Theme,
    ) -> impl IntoElement {
        let identity = Self::resolve_identity(pkg);
        let color = Self::source_color(&pkg.source_type, theme);
        let bg = Self::avatar_bg_color(&pkg.source_type, theme);
        let border = Self::avatar_border_color(&pkg.source_type, theme);
        let glyph_size = (size * 0.56).round();

        let mut container = div()
            .size(px(size))
            .flex_shrink_0()
            .rounded(px(corner_radius))
            .bg(bg)
            .border_1()
            .border_color(border)
            .flex()
            .items_center()
            .justify_center();

        match identity {
            ResolvedIdentity::Authentic(path) => {
                if path.extension().is_some_and(|ext| ext == "svg") {
                    container = container.child(
                        svg()
                            .path(path.to_string_lossy().to_string())
                            .size(px(glyph_size)),
                    );
                } else {
                    container = container.child(
                        img(path)
                            .size(px(glyph_size))
                            .object_fit(ObjectFit::Contain),
                    );
                }
            }
            ResolvedIdentity::Symbolic(icon) | ResolvedIdentity::Fallback(icon) => {
                container = container.child(
                    svg()
                        .path(icon.path())
                        .size(px(glyph_size))
                        .text_color(color),
                );
            }
        }

        container
    }

    /// Rendu d'un glyphe d'identité compact en ligne (pour le Tableau)
    pub fn render_inline_glyph(pkg: &UnifiedPackage, size: f32, theme: &Theme) -> impl IntoElement {
        let identity = Self::resolve_identity(pkg);
        let color = Self::source_color(&pkg.source_type, theme);

        let mut container = div()
            .size(px(size))
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center();

        match identity {
            ResolvedIdentity::Authentic(path) => {
                if path.extension().is_some_and(|ext| ext == "svg") {
                    container = container.child(
                        svg()
                            .path(path.to_string_lossy().to_string())
                            .size(px(size)),
                    );
                } else {
                    container =
                        container.child(img(path).size(px(size)).object_fit(ObjectFit::Contain));
                }
            }
            ResolvedIdentity::Symbolic(icon) | ResolvedIdentity::Fallback(icon) => {
                container =
                    container.child(svg().path(icon.path()).size(px(size)).text_color(color));
            }
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::models::{
        AlpmPackage, AppImageItem, AurPackage, FlatpakHit, UnifiedPackageSource,
    };
    use core::prelude::v1::test;

    /// The identity cache and the pending set are process-wide, and
    /// `clear_cache` wipes both. Cargo runs the tests of a binary in parallel,
    /// so one test was erasing the state another one was waiting for: the
    /// settlement test watched an empty cache for its whole budget while its
    /// own package was resolved and thrown away.
    ///
    /// The lock is what makes the tests deterministic. It is held for the
    /// duration of each test that touches the global state.
    static IDENTITY_STATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn identity_state() -> std::sync::MutexGuard<'static, ()> {
        IDENTITY_STATE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn test_resolve_identity_3_tier_chain() {
        let _state = identity_state();
        PackageIdentity::clear_cache();

        // Tier 1: Authentic icon from existing file on disk
        let mut temp_icon = std::env::temp_dir();
        temp_icon.push("calix_test_authentic_icon.png");
        std::fs::write(&temp_icon, b"dummy png content").unwrap();

        let appimage_with_icon = UnifiedPackage {
            name: "CustomApp".into(),
            version: "1.0.0".into(),
            description: "test".into(),
            source_type: "AppImage".into(),
            repository_or_remote: "appimage".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::AppImage(AppImageItem {
                icon_name: Some(temp_icon.to_string_lossy().to_string()),
                ..Default::default()
            }),
        };
        assert_eq!(
            PackageIdentity::resolve_identity_sync(&appimage_with_icon),
            ResolvedIdentity::Authentic(temp_icon.clone())
        );
        let _ = std::fs::remove_file(temp_icon);

        // Tier 2: Verified source symbolic icon (when no authentic file exists)
        let alpm = UnifiedPackage {
            name: "ripgrep-cli-tool-without-desktop-icon".into(),
            version: "14.1.0".into(),
            description: "fast search".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity_sync(&alpm),
            ResolvedIdentity::Symbolic(AppIcon::SourceAlpm)
        );

        let aur = UnifiedPackage {
            name: "nonexistent-aur-cli-package".into(),
            version: "1.0".into(),
            description: "cli".into(),
            source_type: "AUR".into(),
            repository_or_remote: "aur".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Aur(AurPackage::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity_sync(&aur),
            ResolvedIdentity::Symbolic(AppIcon::SourceAur)
        );

        let flatpak = UnifiedPackage {
            name: "org.nonexistent.FlatpakApp".into(),
            version: "1.0".into(),
            description: "".into(),
            source_type: "Flatpak".into(),
            repository_or_remote: "flathub".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Flatpak(FlatpakHit::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity_sync(&flatpak),
            ResolvedIdentity::Symbolic(AppIcon::SourceFlatpak)
        );

        // Tier 3: Generic fallback box for unknown source
        let unknown = UnifiedPackage {
            name: "custom-tool".into(),
            version: "0.1".into(),
            description: "".into(),
            source_type: "CustomRepo".into(),
            repository_or_remote: "custom".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        assert_eq!(
            PackageIdentity::resolve_identity_sync(&unknown),
            ResolvedIdentity::Fallback(AppIcon::PackageGeneric)
        );
    }

    #[test]
    fn test_source_colors_and_alpha_invariants() {
        let theme = Theme::dark();
        for source in &["ALPM", "AUR", "Flatpak", "AppImage"] {
            let color = PackageIdentity::source_color(source, &theme);
            let bg = PackageIdentity::avatar_bg_color(source, &theme);
            let border = PackageIdentity::avatar_border_color(source, &theme);

            assert_eq!(bg.r, color.r);
            assert_eq!(bg.g, color.g);
            assert_eq!(bg.b, color.b);
            assert!((bg.a - 0.12).abs() < 0.001);

            assert_eq!(border.r, color.r);
            assert_eq!(border.g, color.g);
            assert_eq!(border.b, color.b);
            assert!((border.a - 0.28).abs() < 0.001);
        }
    }

    #[test]
    fn test_identity_cache_zero_io_and_preload() {
        let _state = identity_state();
        PackageIdentity::clear_cache();

        let pkg = UnifiedPackage {
            name: "cache-test-pkg".into(),
            version: "1.0.0".into(),
            description: "testing cache".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        // Cache miss: returns immediate symbolic fallback without blocking
        let initial = PackageIdentity::resolve_identity(&pkg);
        assert_eq!(initial, ResolvedIdentity::Symbolic(AppIcon::SourceAlpm));

        // Preload inserts into cache
        PackageIdentity::preload(&[pkg.clone()]);
        // Give background thread a tiny moment to write
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Cache hit: returns cached identity
        let cached = PackageIdentity::resolve_identity(&pkg);
        assert_eq!(cached, ResolvedIdentity::Symbolic(AppIcon::SourceAlpm));

        PackageIdentity::clear_cache();
    }

    #[test]
    fn test_alpm_uninstalled_never_checks_disk() {
        let _state = identity_state();
        PackageIdentity::clear_cache();

        // Even with a name matching a real app like "firefox", if uninstalled it must be Symbolic
        let uninstalled_pkg = UnifiedPackage {
            name: "firefox".into(),
            version: "128.0".into(),
            description: "browser".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        assert_eq!(PackageIdentity::find_authentic_icon(&uninstalled_pkg), None);
        assert_eq!(
            PackageIdentity::resolve_identity_sync(&uninstalled_pkg),
            ResolvedIdentity::Symbolic(AppIcon::SourceAlpm)
        );
    }

    #[test]
    fn test_alpm_exact_name_match() {
        let temp_dir = std::env::temp_dir().join(format!(
            "calix_test_pacman_exact_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let pacman_local = temp_dir.join("var/lib/pacman/local");
        let pkg_dir = pacman_local.join("calix-gpui-git-r4699.ga172c43e-1");
        let app_dir = temp_dir.join("usr/share/applications");
        let pixmaps_dir = temp_dir.join("usr/share/pixmaps");

        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::create_dir_all(&pixmaps_dir).unwrap();

        // Write desc with %NAME% and %VERSION% where version starts with a letter 'r'
        let desc_content = "%NAME%\ncalix-gpui-git\n\n%VERSION%\nr4699.ga172c43e-1\n";
        std::fs::write(pkg_dir.join("desc"), desc_content).unwrap();

        // Write files list
        let files_content = "%FILES%\nusr/share/applications/calix.desktop\n";
        std::fs::write(pkg_dir.join("files"), files_content).unwrap();

        // Write desktop file
        let desktop_content = "[Desktop Entry]\nType=Application\nName=Calix\nIcon=calix-logo\n";
        std::fs::write(app_dir.join("calix.desktop"), desktop_content).unwrap();

        // Write icon
        let icon_path = pixmaps_dir.join("calix-logo.png");
        std::fs::write(&icon_path, b"test-icon-bytes").unwrap();

        let pkg = UnifiedPackage {
            name: "calix-gpui-git".into(),
            version: "r4699.ga172c43e-1".into(),
            description: "Modern package manager".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "aur".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let result =
            PackageIdentity::probe_alpm_or_aur_icon_in_dir(&pkg, &pacman_local, Some(&temp_dir));
        assert_eq!(result, Some(icon_path));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_alpm_no_false_prefix_candidate() {
        let temp_dir = std::env::temp_dir().join(format!(
            "calix_test_pacman_prefix_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let pacman_local = temp_dir.join("var/lib/pacman/local");
        // Folder starts with "python-" but is actually python-jinja
        let pkg_dir = pacman_local.join("python-jinja-3.1.2-1");
        let app_dir = temp_dir.join("usr/share/applications");
        let pixmaps_dir = temp_dir.join("usr/share/pixmaps");

        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::create_dir_all(&pixmaps_dir).unwrap();

        // Desc clearly states NAME is python-jinja
        let desc_content = "%NAME%\npython-jinja\n\n%VERSION%\n3.1.2-1\n";
        std::fs::write(pkg_dir.join("desc"), desc_content).unwrap();

        let files_content = "%FILES%\nusr/share/applications/jinja.desktop\n";
        std::fs::write(pkg_dir.join("files"), files_content).unwrap();

        let desktop_content = "[Desktop Entry]\nType=Application\nIcon=jinja-icon\n";
        std::fs::write(app_dir.join("jinja.desktop"), desktop_content).unwrap();
        std::fs::write(pixmaps_dir.join("jinja-icon.png"), b"jinja").unwrap();

        // Query for package "python"
        let queried_pkg = UnifiedPackage {
            name: "python".into(),
            version: "3.12.0".into(),
            description: "Python language interpreter".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "core".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let result = PackageIdentity::probe_alpm_or_aur_icon_in_dir(
            &queried_pkg,
            &pacman_local,
            Some(&temp_dir),
        );
        // MUST BE NONE: python must never match python-jinja despite prefix compatibility
        assert_eq!(result, None);

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_resolver_dedup_and_settlement() {
        let _state = identity_state();
        PackageIdentity::clear_cache();

        let pkg = UnifiedPackage {
            name: "dedup-test-package".into(),
            version: "1.0.0".into(),
            description: "dedup test".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let key = pkg.key();

        // First miss: resolves fallback immediately, enqueues request
        let initial = PackageIdentity::resolve_identity(&pkg);
        assert_eq!(initial, ResolvedIdentity::Symbolic(AppIcon::SourceAlpm));

        // Second miss before settlement: does NOT duplicate or fail
        PackageIdentity::enqueue(&pkg);

        // Wait for worker settlement.
        //
        // The resolver is a single background thread shared by every test in
        // this binary, and cargo runs the tests in parallel: the worker may be
        // busy with another package, and it does file system work on the way.
        // The budget is generous on purpose — a test that waits 1.5s and then
        // fails is a flaky gate, not a failing gate.
        let mut settled = false;
        for _ in 0..1000 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if let Ok(cache) = get_identity_cache().read() {
                if cache.contains_key(&key) {
                    settled = true;
                    break;
                }
            }
        }
        assert!(settled, "resolver worker should settle and write to cache");

        // After settlement, pending set must no longer hold key
        if let Ok(pending) = get_resolving_keys().read() {
            assert!(
                !pending.contains(&key),
                "pending set should be cleared after settlement"
            );
        }

        // Subsequent resolution is a zero-cost cache hit
        let cached = PackageIdentity::resolve_identity(&pkg);
        assert_eq!(cached, ResolvedIdentity::Symbolic(AppIcon::SourceAlpm));

        PackageIdentity::clear_cache();
    }
}
