use crate::state::session::PackageViewMode;
use crate::brand::Brand;
use crate::state::session::NavScope;
use crate::visual_style::VisualStyleId;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Paramètres frontend Calix (~/.config/calix/settings.json)
///
/// NOTE D'AUTORITÉ ARCHITECTURALE :
/// `~/.config/calix/settings.json` est le fichier de configuration propre au frontend GPUI.
/// Le backend CLI natif en Zig lit quant à lui `~/.config/calix/config.json` (via `runtime/xdg.zig`).
/// Les champs `calix_icons_enabled`, `calix_search_enabled` et `no_confirm` sont des champs
/// de compatibilité conservés pour la fidélité du schéma Serde avec les versions antérieures,
/// tandis que le frontend GPUI applique ses options de mutation (`--ui-mode`, `--no-confirm`,
/// `--cascade`, `--remove-config`) via les arguments passés au binaire CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CalixSettings {
    #[serde(default = "default_true")]
    pub aur_enabled: bool,
    #[serde(default = "default_true")]
    pub app_image_enabled: bool,
    #[serde(default = "default_true")]
    pub flat_pack_enabled: bool,
    #[serde(default = "default_true")]
    pub calix_icons_enabled: bool,
    #[serde(default = "default_true")]
    pub calix_search_enabled: bool,
    #[serde(default)]
    pub no_confirm: bool,
    #[serde(default)]
    pub package_management_cascade_delete: bool,
    #[serde(default)]
    pub package_management_remove_configs: bool,
}

fn default_true() -> bool {
    true
}

impl Default for CalixSettings {
    fn default() -> Self {
        Self {
            aur_enabled: true,
            app_image_enabled: true,
            flat_pack_enabled: true,
            calix_icons_enabled: true,
            calix_search_enabled: true,
            no_confirm: true,
            package_management_cascade_delete: true,
            package_management_remove_configs: true,
        }
    }
}

/// Paramètres UI spécifiques à GPUI (~/.config/calix/gpui-ui.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuiUiConfig {
    #[serde(default = "default_dark_theme")]
    pub dark_theme: bool,
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    #[serde(default = "default_compact_view")]
    pub compact_view: bool,
    #[serde(default = "default_log_drawer_open")]
    pub log_drawer_open: bool,
    #[serde(default = "default_log_drawer_height")]
    pub log_drawer_height: f32,
    /// Le dernier périmètre de la surface des paquets, retrouvé au lancement.
    #[serde(default = "default_last_scope")]
    pub last_scope: NavScope,
    #[serde(default)]
    pub reduce_motion: bool,
    #[serde(default = "default_view_mode")]
    pub view_mode: PackageViewMode,
    #[serde(default = "default_visual_style")]
    pub visual_style: VisualStyleId,
    /// La couleur de marque : l'accent de l'interface et la couleur de la
    /// coupe en écalille. C'est l'utilisateur qui choisit.
    #[serde(default = "default_brand_color")]
    pub brand_color: Brand,
}

/// Le plancher de la fenêtre, en pixels logiques.
///
/// C'est une *cible de qualification*, pas un chiffre arbitraire : la
/// composition doit y tenir — la colonne de navigation, le catalogue, les
/// commandes essentielles.
///
/// Le budget, mesuré, et il s'additionne :
///
/// ```
/// 495  la ligne du catalogue : ses cinq colonnes fixes et son padding
/// + 22 le glyphe d'identité, son écart, et le nom du p90 des paquets réels
/// + 143 le nom, pour 90 % des paquets de la machine (143 px de texte)
/// = 660 le contenu que le catalogue garde
/// + 56  la colonne de navigation, repliée
/// = 716  arrondi à 720
/// ```
///
/// La colonne de navigation se replie d'elle-même quand la place manque
/// (l'échelle du contrat spatial), donc elle ne coûte pas 190 px au
/// plancher : elle coûte 56.
///
/// Un plancher est une *demande* : un compositeur à tuiles peut accorder
/// moins. La composition doit donc tenir aussi en dessous, et c'est ce que
/// le banc mesure.
pub const MIN_WINDOW_WIDTH: f32 = 720.0;
pub const MIN_WINDOW_HEIGHT: f32 = 480.0;

fn default_dark_theme() -> bool {
    true
}

fn default_window_width() -> f32 {
    1280.0
}

fn default_window_height() -> f32 {
    840.0
}

fn default_compact_view() -> bool {
    false
}

fn default_log_drawer_open() -> bool {
    false
}

fn default_log_drawer_height() -> f32 {
    220.0
}

fn default_last_scope() -> NavScope {
    NavScope::default()
}

fn default_view_mode() -> PackageViewMode {
    PackageViewMode::Table
}

fn default_visual_style() -> VisualStyleId {
    VisualStyleId::Standard
}

fn default_brand_color() -> Brand {
    Brand::default()
}

impl Default for GpuiUiConfig {
    fn default() -> Self {
        Self {
            dark_theme: default_dark_theme(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            compact_view: default_compact_view(),
            log_drawer_open: default_log_drawer_open(),
            log_drawer_height: default_log_drawer_height(),
            last_scope: default_last_scope(),
            reduce_motion: false,
            view_mode: default_view_mode(),
            visual_style: default_visual_style(),
            brand_color: default_brand_color(),
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .map(|p| p.join("calix"))
            .unwrap_or_else(|| PathBuf::from(".config/calix"))
    }

    pub fn sanitize_window_size(w: f32, h: f32) -> (f32, f32) {
        let width = if w.is_nan() || w < MIN_WINDOW_WIDTH {
            default_window_width()
        } else {
            w
        };
        let height = if h.is_nan() || h < MIN_WINDOW_HEIGHT {
            default_window_height()
        } else {
            h
        };
        (width, height)
    }

    /// Writes file atomically via sibling temporary file, fsync, atomic rename, and parent dir fsync.
    /// In case of error prior to successful rename, cleans up the temporary file.
    pub fn atomic_write_file(target: &Path, content: &str) -> Result<()> {
        let parent = target
            .parent()
            .context("Target path must have a parent directory")?;
        fs::create_dir_all(parent)?;

        let file_name = target
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let temp_name = format!(
            ".{}.tmp.{}.{}",
            file_name,
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let temp_path = parent.join(temp_name);

        let write_res = (|| -> Result<()> {
            use std::io::Write;
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(content.as_bytes())?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_res {
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }

        if let Err(e) = fs::rename(&temp_path, target) {
            let _ = fs::remove_file(&temp_path);
            return Err(e.into());
        }

        // Fsync the parent directory to ensure directory entry durability across crashes
        if let Ok(dir) = fs::File::open(parent) {
            let _ = dir.sync_all();
        }

        Ok(())
    }

    pub fn load_calix_settings() -> CalixSettings {
        let path = Self::config_dir().join("settings.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<CalixSettings>(&content) {
                    return settings;
                }
            }
        }
        CalixSettings::default()
    }

    pub fn save_calix_settings(settings: &CalixSettings) -> Result<()> {
        let dir = Self::config_dir();
        let path = dir.join("settings.json");
        let content = serde_json::to_string_pretty(settings)?;
        Self::atomic_write_file(&path, &content)?;
        Ok(())
    }

    pub fn load_gpui_config() -> GpuiUiConfig {
        let path = Self::config_dir().join("gpui-ui.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<GpuiUiConfig>(&content) {
                    return config;
                }
            }
        }
        GpuiUiConfig::default()
    }

    pub fn load_gpui_config_sanitized() -> GpuiUiConfig {
        let mut config = Self::load_gpui_config();
        let (w, h) = Self::sanitize_window_size(config.window_width, config.window_height);
        config.window_width = w;
        config.window_height = h;
        config
    }

    pub fn save_gpui_config(config: &GpuiUiConfig) -> Result<()> {
        let dir = Self::config_dir();
        let path = dir.join("gpui-ui.json");
        let content = serde_json::to_string_pretty(config)?;
        Self::atomic_write_file(&path, &content)?;
        Ok(())
    }

    /// List all supported typed frontend settings
    pub fn list_settings() -> Vec<SettingEntry> {
        let gpui = Self::load_gpui_config();
        let calix = Self::load_calix_settings();

        vec![
            SettingEntry {
                key: "theme".to_string(),
                value: if gpui.dark_theme { "dark" } else { "light" }.to_string(),
                default: "dark".to_string(),
                description: "Application color theme ('dark' or 'light')".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "brand-color".to_string(),
                value: gpui.brand_color.as_str().to_string(),
                default: Brand::default().as_str().to_string(),
                description: "Brand colour of the shell mark and the interface accent ('orange' or 'royal')".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "compact-view".to_string(),
                value: gpui.compact_view.to_string(),
                default: "false".to_string(),
                description: "Compact sidebar and workstation card density".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "reduce-motion".to_string(),
                value: gpui.reduce_motion.to_string(),
                default: "false".to_string(),
                description: "Disable animated transitions across the UI".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "view-mode".to_string(),
                value: match gpui.view_mode {
                    PackageViewMode::Table => "table",
                    PackageViewMode::Cards => "cards",
                }
                .to_string(),
                default: "table".to_string(),
                description: "Default package workstation display surface ('table' or 'cards')"
                    .to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "log-drawer-open".to_string(),
                value: gpui.log_drawer_open.to_string(),
                default: "false".to_string(),
                description: "Keep bottom operation log drawer open by default".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "log-drawer-height".to_string(),
                value: format!("{:.1}", gpui.log_drawer_height),
                default: "220.0".to_string(),
                description: "Height of operation log drawer in pixels (120.0 - 600.0)".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "aur-enabled".to_string(),
                value: calix.aur_enabled.to_string(),
                default: "true".to_string(),
                description: "Enable Arch User Repository (AUR) package backend".to_string(),
                authority: "settings".to_string(),
            },
            SettingEntry {
                key: "flatpak-enabled".to_string(),
                value: calix.flat_pack_enabled.to_string(),
                default: "true".to_string(),
                description: "Enable Flatpak application backend".to_string(),
                authority: "settings".to_string(),
            },
            SettingEntry {
                key: "appimage-enabled".to_string(),
                value: calix.app_image_enabled.to_string(),
                default: "true".to_string(),
                description: "Enable AppImage standalone application backend".to_string(),
                authority: "settings".to_string(),
            },
            SettingEntry {
                key: "cascade-delete".to_string(),
                value: calix.package_management_cascade_delete.to_string(),
                default: "true".to_string(),
                description: "Cascade deletion of unneeded dependencies on removal".to_string(),
                authority: "settings".to_string(),
            },
            SettingEntry {
                key: "remove-configs".to_string(),
                value: calix.package_management_remove_configs.to_string(),
                default: "true".to_string(),
                description: "Purge configuration files when removing packages".to_string(),
                authority: "settings".to_string(),
            },
            SettingEntry {
                key: "window-width".to_string(),
                value: format!("{:.1}", gpui.window_width),
                default: "1280.0".to_string(),
                description: "Restored application window width (minimum 1024.0)".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "window-height".to_string(),
                value: format!("{:.1}", gpui.window_height),
                default: "840.0".to_string(),
                description: "Restored application window height (minimum 680.0)".to_string(),
                authority: "gpui-ui".to_string(),
            },
            SettingEntry {
                key: "visual-style".to_string(),
                value: gpui.visual_style.as_str().to_string(),
                default: "standard".to_string(),
                description: "Selectable visual style profile ('standard' or 'transparency')"
                    .to_string(),
                authority: "gpui-ui".to_string(),
            },
        ]
    }

    /// Retrieve the current string value of a setting
    pub fn get_setting(key: &str) -> Result<String> {
        let entries = Self::list_settings();
        for entry in entries {
            if entry.key.eq_ignore_ascii_case(key) {
                return Ok(entry.value);
            }
        }
        anyhow::bail!("Unknown setting key '{}'", key)
    }

    /// Set and validate a setting key to a new string value
    pub fn set_setting(key: &str, value: &str) -> Result<()> {
        let key_lower = key.to_ascii_lowercase();
        match key_lower.as_str() {
            "theme" => {
                let mut config = Self::load_gpui_config();
                match value.to_ascii_lowercase().as_str() {
                    "dark" => config.dark_theme = true,
                    "light" => config.dark_theme = false,
                    other => anyhow::bail!(
                        "Invalid theme value '{}', expected 'dark' or 'light'",
                        other
                    ),
                }
                Self::save_gpui_config(&config)?;
            }
            "brand-color" => {
                let mut config = Self::load_gpui_config();
                let brand = Brand::parse(value).map_err(|e| anyhow::anyhow!("{e}"))?;
                config.brand_color = brand;
                Self::save_gpui_config(&config)?;
            }
            "compact-view" => {
                let mut config = Self::load_gpui_config();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for compact-view, expected 'true' or 'false'",
                        value
                    )
                })?;
                config.compact_view = val;
                Self::save_gpui_config(&config)?;
            }
            "reduce-motion" => {
                let mut config = Self::load_gpui_config();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for reduce-motion, expected 'true' or 'false'",
                        value
                    )
                })?;
                config.reduce_motion = val;
                Self::save_gpui_config(&config)?;
            }
            "view-mode" => {
                let mut config = Self::load_gpui_config();
                match value.to_ascii_lowercase().as_str() {
                    "table" => config.view_mode = PackageViewMode::Table,
                    "cards" => config.view_mode = PackageViewMode::Cards,
                    other => {
                        anyhow::bail!("Invalid view-mode '{}', expected 'table' or 'cards'", other)
                    }
                }
                Self::save_gpui_config(&config)?;
            }
            "log-drawer-open" => {
                let mut config = Self::load_gpui_config();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for log-drawer-open, expected 'true' or 'false'",
                        value
                    )
                })?;
                config.log_drawer_open = val;
                Self::save_gpui_config(&config)?;
            }
            "log-drawer-height" => {
                let mut config = Self::load_gpui_config();
                let val: f32 = value.parse().map_err(|_| {
                    anyhow::anyhow!("Invalid number '{}' for log-drawer-height", value)
                })?;
                if !(120.0..=600.0).contains(&val) {
                    anyhow::bail!(
                        "log-drawer-height must be between 120.0 and 600.0, got '{}'",
                        val
                    );
                }
                config.log_drawer_height = val;
                Self::save_gpui_config(&config)?;
            }
            "aur-enabled" => {
                let mut settings = Self::load_calix_settings();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for aur-enabled, expected 'true' or 'false'",
                        value
                    )
                })?;
                settings.aur_enabled = val;
                Self::save_calix_settings(&settings)?;
            }
            "flatpak-enabled" => {
                let mut settings = Self::load_calix_settings();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for flatpak-enabled, expected 'true' or 'false'",
                        value
                    )
                })?;
                settings.flat_pack_enabled = val;
                Self::save_calix_settings(&settings)?;
            }
            "appimage-enabled" => {
                let mut settings = Self::load_calix_settings();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for appimage-enabled, expected 'true' or 'false'",
                        value
                    )
                })?;
                settings.app_image_enabled = val;
                Self::save_calix_settings(&settings)?;
            }
            "cascade-delete" => {
                let mut settings = Self::load_calix_settings();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for cascade-delete, expected 'true' or 'false'",
                        value
                    )
                })?;
                settings.package_management_cascade_delete = val;
                Self::save_calix_settings(&settings)?;
            }
            "remove-configs" => {
                let mut settings = Self::load_calix_settings();
                let val: bool = value.parse().map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid boolean '{}' for remove-configs, expected 'true' or 'false'",
                        value
                    )
                })?;
                settings.package_management_remove_configs = val;
                Self::save_calix_settings(&settings)?;
            }
            "window-width" => {
                let mut config = Self::load_gpui_config();
                let val: f32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid number '{}' for window-width", value))?;
                if val < MIN_WINDOW_WIDTH {
                    anyhow::bail!(
                        "window-width must be at least {}, got '{}'",
                        MIN_WINDOW_WIDTH,
                        val
                    );
                }
                config.window_width = val;
                Self::save_gpui_config(&config)?;
            }
            "visual-style" => {
                let mut config = Self::load_gpui_config();
                config.visual_style =
                    VisualStyleId::parse(value).map_err(|e| anyhow::anyhow!(e))?;
                Self::save_gpui_config(&config)?;
            }
            "window-height" => {
                let mut config = Self::load_gpui_config();
                let val: f32 = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("Invalid number '{}' for window-height", value))?;
                if val < MIN_WINDOW_HEIGHT {
                    anyhow::bail!(
                        "window-height must be at least {}, got '{}'",
                        MIN_WINDOW_HEIGHT,
                        val
                    );
                }
                config.window_height = val;
                Self::save_gpui_config(&config)?;
            }
            other => anyhow::bail!("Unknown setting key '{}'", other),
        }
        Ok(())
    }

    /// Reset a specific setting key or all settings to defaults
    pub fn reset_setting(key: &str) -> Result<()> {
        let key_lower = key.to_ascii_lowercase();
        match key_lower.as_str() {
            "all" => Self::reset_all(),
            "theme" => Self::set_setting("theme", "dark"),
            "brand-color" => Self::set_setting(
                "brand-color",
                Brand::default().as_str(),
            ),
            "compact-view" => Self::set_setting("compact-view", "false"),
            "reduce-motion" => Self::set_setting("reduce-motion", "false"),
            "view-mode" => Self::set_setting("view-mode", "table"),
            "log-drawer-open" => Self::set_setting("log-drawer-open", "false"),
            "log-drawer-height" => Self::set_setting("log-drawer-height", "220.0"),
            "aur-enabled" => Self::set_setting("aur-enabled", "true"),
            "flatpak-enabled" => Self::set_setting("flatpak-enabled", "true"),
            "appimage-enabled" => Self::set_setting("appimage-enabled", "true"),
            "cascade-delete" => Self::set_setting("cascade-delete", "true"),
            "remove-configs" => Self::set_setting("remove-configs", "true"),
            "window-width" => Self::set_setting("window-width", "1280.0"),
            "window-height" => Self::set_setting("window-height", "840.0"),
            "visual-style" => Self::set_setting("visual-style", "standard"),
            other => anyhow::bail!("Unknown setting key '{}'", other),
        }
    }

    /// Resets all settings across both configurations to defaults.
    /// Provides crash-safe atomic replacement per configuration file (not a multi-file POSIX transaction).
    pub fn reset_all() -> Result<()> {
        Self::save_calix_settings(&CalixSettings::default())?;
        Self::save_gpui_config(&GpuiUiConfig::default())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
    pub default: String,
    pub description: String,
    pub authority: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_gpui_config_all_fields_default_on_empty_json() {
        let empty_json = "{}";
        let parsed: GpuiUiConfig = serde_json::from_str(empty_json)
            .expect("Empty JSON object should deserialize with default values for all fields");

        assert!(parsed.dark_theme);
        assert_eq!(parsed.window_width, 1280.0);
        assert_eq!(parsed.window_height, 840.0);
        assert!(!parsed.compact_view);
        assert!(!parsed.log_drawer_open);
        assert_eq!(parsed.log_drawer_height, 220.0);
        assert_eq!(parsed.last_scope, NavScope::Results);
        assert!(!parsed.reduce_motion);
        assert_eq!(parsed.view_mode, PackageViewMode::Table);
        assert_eq!(parsed.visual_style, VisualStyleId::Standard);
        assert_eq!(parsed.brand_color, Brand::Orange);
    }

    #[test]
    fn test_gpui_config_preserves_explicit_brand_color() {
        let royal_json = r#"{"brand_color": "royal"}"#;
        let parsed: GpuiUiConfig =
            serde_json::from_str(royal_json).expect("Should deserialize the royal colourway");
        assert_eq!(parsed.brand_color, Brand::Royal);

        let orange_json = r#"{"brand_color": "orange"}"#;
        let parsed: GpuiUiConfig =
            serde_json::from_str(orange_json).expect("Should deserialize the orange colourway");
        assert_eq!(parsed.brand_color, Brand::Orange);
    }

    #[test]
    fn the_brand_colour_is_part_of_the_settings_registry() {
        let entries = ConfigManager::list_settings();
        let brand = entries
            .iter()
            .find(|entry| entry.key == "brand-color")
            .expect("the brand colour is a registered setting");
        assert_eq!(brand.default, Brand::default().as_str());
        assert_eq!(brand.authority, "gpui-ui");
    }

    #[test]
    fn test_gpui_config_preserves_explicit_view_mode() {
        let cards_json = r#"{"view_mode": "Cards"}"#;
        let parsed: GpuiUiConfig =
            serde_json::from_str(cards_json).expect("Should deserialize cards");
        assert_eq!(parsed.view_mode, PackageViewMode::Cards);

        let table_json = r#"{"view_mode": "Table"}"#;
        let parsed: GpuiUiConfig =
            serde_json::from_str(table_json).expect("Should deserialize table");
        assert_eq!(parsed.view_mode, PackageViewMode::Table);
    }

    #[test]
    fn test_gpui_config_preserves_explicit_visual_style() {
        let transparency_json = r#"{"visual_style": "transparency"}"#;
        let parsed: GpuiUiConfig =
            serde_json::from_str(transparency_json).expect("Should deserialize transparency");
        assert_eq!(parsed.visual_style, VisualStyleId::Transparency);

        let standard_json = r#"{"visual_style": "standard"}"#;
        let parsed: GpuiUiConfig =
            serde_json::from_str(standard_json).expect("Should deserialize standard");
        assert_eq!(parsed.visual_style, VisualStyleId::Standard);
    }

    #[test]
    fn test_sanitize_window_size_enforces_minimums() {
        // Below minimum falls back to default
        let (w, h) = ConfigManager::sanitize_window_size(400.0, 300.0);
        assert_eq!(w, 1280.0);
        assert_eq!(h, 840.0);

        // NaN falls back to default
        let (w_nan, h_nan) = ConfigManager::sanitize_window_size(f32::NAN, f32::NAN);
        assert_eq!(w_nan, 1280.0);
        assert_eq!(h_nan, 840.0);

        // Valid size >= minimum is preserved
        let (w_valid, h_valid) = ConfigManager::sanitize_window_size(1600.0, 1000.0);
        assert_eq!(w_valid, 1600.0);
        assert_eq!(h_valid, 1000.0);

        // Boundary test at exact minimums
        let (w_min, h_min) =
            ConfigManager::sanitize_window_size(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT);
        assert_eq!(w_min, 720.0);
        assert_eq!(h_min, 480.0);
    }

    #[test]
    fn test_gpui_config_backward_compatibility_without_reduce_motion() {
        let legacy_json = r#"{
            "dark_theme": true,
            "window_width": 1280.0,
            "window_height": 840.0,
            "compact_view": true,
            "log_drawer_open": true,
            "log_drawer_height": 250.0,
            "last_scope": "installed"
        }"#;

        let parsed: GpuiUiConfig = serde_json::from_str(legacy_json)
            .expect("Legacy config without reduce_motion should deserialize successfully");

        assert!(parsed.dark_theme);
        assert_eq!(parsed.window_width, 1280.0);
        assert_eq!(parsed.window_height, 840.0);
        assert!(parsed.compact_view);
        assert!(parsed.log_drawer_open);
        assert_eq!(parsed.log_drawer_height, 250.0);
        assert_eq!(parsed.last_scope, NavScope::Installed);
        assert!(
            !parsed.reduce_motion,
            "Default for reduce_motion must be false"
        );
        assert_eq!(
            parsed.visual_style,
            VisualStyleId::Standard,
            "Legacy config without visual_style must default to Standard"
        );
    }

    #[test]
    fn test_gpui_config_with_explicit_reduce_motion() {
        let json_with_reduce = r#"{
            "dark_theme": false,
            "window_width": 1024.0,
            "window_height": 768.0,
            "compact_view": false,
            "log_drawer_open": false,
            "log_drawer_height": 200.0,
            "last_scope": "results",
            "reduce_motion": true
        }"#;

        let parsed: GpuiUiConfig = serde_json::from_str(json_with_reduce)
            .expect("Config with reduce_motion should deserialize successfully");

        assert!(parsed.reduce_motion);
    }

    #[test]
    fn test_list_settings_completeness() {
        let list = ConfigManager::list_settings();
        let expected_keys = [
            "theme",
            "brand-color",
            "compact-view",
            "reduce-motion",
            "view-mode",
            "log-drawer-open",
            "log-drawer-height",
            "aur-enabled",
            "flatpak-enabled",
            "appimage-enabled",
            "cascade-delete",
            "remove-configs",
            "window-width",
            "window-height",
            "visual-style",
        ];
        assert_eq!(list.len(), expected_keys.len());
        for key in expected_keys {
            assert!(
                list.iter().any(|entry| entry.key == key),
                "Missing expected setting key: {}",
                key
            );
        }
    }

    #[test]
    fn test_get_setting_keys() {
        assert!(ConfigManager::get_setting("theme").is_ok());
        assert!(ConfigManager::get_setting("view-mode").is_ok());
        assert!(ConfigManager::get_setting("compact-view").is_ok());
        assert!(ConfigManager::get_setting("visual-style").is_ok());
        assert!(ConfigManager::get_setting("brand-color").is_ok());
        assert!(ConfigManager::get_setting("non_existent_key").is_err());
    }

    #[test]
    fn test_setting_validation_errors() {
        assert!(ConfigManager::set_setting("theme", "neon").is_err());
        assert!(ConfigManager::set_setting("view-mode", "grid").is_err());
        assert!(ConfigManager::set_setting("compact-view", "not_a_bool").is_err());
        assert!(ConfigManager::set_setting("log-drawer-height", "50.0").is_err());
        assert!(ConfigManager::set_setting("log-drawer-height", "1000.0").is_err());
        assert!(ConfigManager::set_setting("window-width", "500.0").is_err());
        assert!(ConfigManager::set_setting("window-height", "400.0").is_err());
        assert!(ConfigManager::set_setting("visual-style", "neon").is_err());
        assert!(ConfigManager::set_setting("brand-color", "cyan").is_err());
        assert!(ConfigManager::set_setting("invalid_setting_key", "val").is_err());
    }

    #[test]
    fn test_atomic_write_file_integrity() {
        let test_dir = ConfigManager::config_dir().join(".test_atomic");
        let _ = std::fs::remove_dir_all(&test_dir);
        let _ = std::fs::create_dir_all(&test_dir);
        let target_file = test_dir.join("test_write.json");

        let sample_content = r#"{"test": "atomic_content_guarantee"}"#;
        let res = ConfigManager::atomic_write_file(&target_file, sample_content);
        assert!(res.is_ok());

        let read_back = std::fs::read_to_string(&target_file).expect("File should exist");
        assert_eq!(read_back, sample_content);

        let entries = std::fs::read_dir(&test_dir).unwrap().collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);

        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
