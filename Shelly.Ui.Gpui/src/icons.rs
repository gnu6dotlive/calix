use gpui::{AssetSource, Result, SharedString};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppIcon {
    Browse,
    Search,
    Installed,
    Updates,
    News,
    Settings,
    Overview,
    Dependencies,
    FilesBuild,
    Cards,
    Table,
    Copy,
    ExternalUrl,
    FilePath,
    Collapse,
    Expand,
    Trash,
    Calix,
    Check,
    Close,
    Info,
    Warning,
    SourceAlpm,
    SourceAur,
    SourceFlatpak,
    SourceAppImage,
    SourceCargo,
    SourceNpm,
    SourceRelease,
    PackageGeneric,
    RenderLab,
}

impl AppIcon {
    pub const ALL: &'static [AppIcon] = &[
        AppIcon::Browse,
        AppIcon::Search,
        AppIcon::Installed,
        AppIcon::Updates,
        AppIcon::News,
        AppIcon::Settings,
        AppIcon::Overview,
        AppIcon::Dependencies,
        AppIcon::FilesBuild,
        AppIcon::Cards,
        AppIcon::Table,
        AppIcon::Copy,
        AppIcon::ExternalUrl,
        AppIcon::FilePath,
        AppIcon::Collapse,
        AppIcon::Expand,
        AppIcon::Trash,
        AppIcon::Calix,
        AppIcon::Check,
        AppIcon::Close,
        AppIcon::Info,
        AppIcon::Warning,
        AppIcon::SourceAlpm,
        AppIcon::SourceAur,
        AppIcon::SourceFlatpak,
        AppIcon::SourceAppImage,
        AppIcon::SourceCargo,
        AppIcon::SourceNpm,
        AppIcon::SourceRelease,
        AppIcon::PackageGeneric,
        AppIcon::RenderLab,
    ];

    pub fn path(self) -> &'static str {
        match self {
            AppIcon::Browse => "icons/browse.svg",
            AppIcon::Search => "icons/search.svg",
            AppIcon::Installed => "icons/installed.svg",
            AppIcon::Updates => "icons/updates.svg",
            AppIcon::News => "icons/news.svg",
            AppIcon::Settings => "icons/settings.svg",
            AppIcon::Overview => "icons/overview.svg",
            AppIcon::Dependencies => "icons/dependencies.svg",
            AppIcon::FilesBuild => "icons/files-build.svg",
            AppIcon::Cards => "icons/cards.svg",
            AppIcon::Table => "icons/table.svg",
            AppIcon::Copy => "icons/copy.svg",
            AppIcon::ExternalUrl => "icons/external-url.svg",
            AppIcon::FilePath => "icons/file-path.svg",
            AppIcon::Collapse => "icons/collapse.svg",
            AppIcon::Expand => "icons/expand.svg",
            AppIcon::Trash => "icons/trash.svg",
            AppIcon::Calix => "icons/calix.svg",
            AppIcon::Check => "icons/check.svg",
            AppIcon::Close => "icons/close.svg",
            AppIcon::Info => "icons/info.svg",
            AppIcon::Warning => "icons/warning.svg",
            AppIcon::SourceAlpm => "icons/source-alpm.svg",
            AppIcon::SourceAur => "icons/source-aur.svg",
            AppIcon::SourceFlatpak => "icons/source-flatpak.svg",
            AppIcon::SourceAppImage => "icons/source-appimage.svg",
            AppIcon::SourceCargo => "icons/source-cargo.svg",
            AppIcon::SourceNpm => "icons/source-npm.svg",
            AppIcon::SourceRelease => "icons/source-release.svg",
            AppIcon::PackageGeneric => "icons/package-generic.svg",
            AppIcon::RenderLab => "icons/render-lab.svg",
        }
    }
}

#[derive(Default)]
pub struct AppIcons;

impl AppIcons {
    pub fn new() -> Self {
        Self
    }
}

impl AssetSource for AppIcons {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let bytes: Option<&'static [u8]> = match path {
            "icons/browse.svg" => Some(include_bytes!("icons/browse.svg")),
            "icons/search.svg" => Some(include_bytes!("icons/search.svg")),
            "icons/installed.svg" => Some(include_bytes!("icons/installed.svg")),
            "icons/updates.svg" => Some(include_bytes!("icons/updates.svg")),
            "icons/news.svg" => Some(include_bytes!("icons/news.svg")),
            "icons/settings.svg" => Some(include_bytes!("icons/settings.svg")),
            "icons/overview.svg" => Some(include_bytes!("icons/overview.svg")),
            "icons/dependencies.svg" => Some(include_bytes!("icons/dependencies.svg")),
            "icons/files-build.svg" => Some(include_bytes!("icons/files-build.svg")),
            "icons/cards.svg" => Some(include_bytes!("icons/cards.svg")),
            "icons/table.svg" => Some(include_bytes!("icons/table.svg")),
            "icons/copy.svg" => Some(include_bytes!("icons/copy.svg")),
            "icons/external-url.svg" => Some(include_bytes!("icons/external-url.svg")),
            "icons/file-path.svg" => Some(include_bytes!("icons/file-path.svg")),
            "icons/collapse.svg" => Some(include_bytes!("icons/collapse.svg")),
            "icons/expand.svg" => Some(include_bytes!("icons/expand.svg")),
            "icons/trash.svg" => Some(include_bytes!("icons/trash.svg")),
            "icons/calix.svg" => Some(include_bytes!("icons/calix.svg")),
            "icons/check.svg" => Some(include_bytes!("icons/check.svg")),
            "icons/close.svg" => Some(include_bytes!("icons/close.svg")),
            "icons/info.svg" => Some(include_bytes!("icons/info.svg")),
            "icons/warning.svg" => Some(include_bytes!("icons/warning.svg")),
            "icons/source-alpm.svg" => Some(include_bytes!("icons/source-alpm.svg")),
            "icons/source-aur.svg" => Some(include_bytes!("icons/source-aur.svg")),
            "icons/source-flatpak.svg" => Some(include_bytes!("icons/source-flatpak.svg")),
            "icons/source-appimage.svg" => Some(include_bytes!("icons/source-appimage.svg")),
            "icons/source-cargo.svg" => Some(include_bytes!("icons/source-cargo.svg")),
            "icons/source-npm.svg" => Some(include_bytes!("icons/source-npm.svg")),
            "icons/source-release.svg" => Some(include_bytes!("icons/source-release.svg")),
            "icons/package-generic.svg" => Some(include_bytes!("icons/package-generic.svg")),
            "icons/render-lab.svg" => Some(include_bytes!("icons/render-lab.svg")),
            _ => None,
        };
        Ok(bytes.map(Cow::Borrowed))
    }

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(AppIcon::ALL
            .iter()
            .map(|icon| SharedString::from(icon.path()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_app_icons_resolve_embedded_svg() {
        let assets = AppIcons::new();
        for icon in AppIcon::ALL {
            let path = icon.path();
            let loaded = assets.load(path).expect("Failed to load icon");
            assert!(
                loaded.is_some(),
                "Icon {:?} at path '{}' must resolve to Some bytes",
                icon,
                path
            );
            let bytes = loaded.unwrap();
            assert!(
                !bytes.is_empty(),
                "Icon {:?} at path '{}' must not be empty",
                icon,
                path
            );
            let is_svg = bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml");
            assert!(
                is_svg,
                "Icon {:?} at path '{}' must be valid SVG content",
                icon, path
            );
        }
    }

    #[test]
    fn test_unknown_icon_returns_none() {
        let assets = AppIcons::new();
        let loaded = assets
            .load("icons/non_existent.svg")
            .expect("Asset loading should not error");
        assert!(loaded.is_none());
    }
}
