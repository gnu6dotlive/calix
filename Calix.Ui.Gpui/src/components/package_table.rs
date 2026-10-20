use crate::backend::models::{UnifiedPackage, UnifiedPackageSource};
use crate::components::package_identity::PackageIdentity;
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct PackageTable;

impl PackageTable {
    pub fn format_bytes(bytes: u64) -> String {
        const KIB: u64 = 1024;
        const MIB: u64 = 1024 * 1024;
        const GIB: u64 = 1024 * 1024 * 1024;

        if bytes >= GIB {
            format!("{:.1} GiB", bytes as f64 / GIB as f64)
        } else if bytes >= MIB {
            format!("{:.1} MiB", bytes as f64 / MIB as f64)
        } else if bytes >= KIB {
            format!("{:.1} KiB", bytes as f64 / KIB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    pub fn display_size(pkg: &UnifiedPackage) -> String {
        let opt_size = match &pkg.inner {
            UnifiedPackageSource::Standard(alpm) => {
                alpm.installed_size.or(alpm.download_size).or(alpm.size)
            }
            UnifiedPackageSource::Flatpak(fp) => fp.installed_size.or(fp.download_size),
            UnifiedPackageSource::AppImage(ai) => ai.size_on_disk,
            UnifiedPackageSource::Aur(_) => None,
            // Les écosystèmes Calix n'exposent pas de taille d'installation.
            UnifiedPackageSource::Cargo(_)
            | UnifiedPackageSource::Npm(_)
            | UnifiedPackageSource::Release(_) => None,
        };

        match opt_size {
            Some(s) if s > 0 => Self::format_bytes(s),
            _ => "—".to_string(),
        }
    }

    pub fn render_header(theme: &Theme) -> impl IntoElement {
        div()
            .h(px(UiMetrics::ROW_HEIGHT_HEADER))
            .w_full()
            .flex()
            .items_center()
            .px_3()
            .bg(theme.bg_surface)
            .border_b_1()
            .border_color(theme.border)
            .text_xs()
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text_muted)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child("NAME"),
            )
            .child(
                div()
                    .w(px(105.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child("VERSION"),
            )
            .child(
                div()
                    .w(px(105.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child("SOURCE"),
            )
            .child(
                div()
                    .w(px(80.0))
                    .flex()
                    .justify_end()
                    .pr_3()
                    .overflow_hidden()
                    .text_ellipsis()
                    .child("SIZE"),
            )
            .child(
                div()
                    .w(px(85.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child("STATUS"),
            )
    }

    pub fn render_row(
        pkg: &UnifiedPackage,
        is_selected: bool,
        theme: &Theme,
        compact: bool,
    ) -> impl IntoElement {
        let bg_color = if is_selected {
            theme.bg_surface_active
        } else {
            theme.bg_surface
        };

        let border_color = if is_selected {
            theme.border_focus
        } else {
            theme.border
        };

        let row_height = if compact {
            UiMetrics::ROW_HEIGHT_COMPACT
        } else {
            UiMetrics::ROW_HEIGHT_NORMAL
        };

        // Source et repo textuels sans pills
        let source_label = match pkg.source_type.to_uppercase().as_str() {
            "ALPM" => "Arch",
            "AUR" => "AUR",
            "FLATPAK" => "Flatpak",
            "APPIMAGE" => "AppImage",
            _ => pkg.source_type.as_str(),
        };

        let source_display = if !pkg.repository_or_remote.is_empty()
            && !pkg
                .repository_or_remote
                .eq_ignore_ascii_case(&pkg.source_type)
            && !pkg.repository_or_remote.eq_ignore_ascii_case(source_label)
        {
            format!("{} / {}", source_label, pkg.repository_or_remote)
        } else {
            source_label.to_string()
        };

        div()
            .h(px(row_height))
            .w_full()
            .flex()
            .items_center()
            .px_3()
            .border_b_1()
            .border_color(border_color)
            .bg(bg_color)
            .cursor_pointer()
            .overflow_hidden()
            .when(is_selected, |el| el.border_l_4().border_color(theme.accent))
            .when(!is_selected, |el| {
                let hover_bg = theme.bg_surface_hover;
                el.hover(move |s| s.bg(hover_bg))
            })
            // Column 1: Name avec icône d'identité en ligne
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_2()
                    .overflow_hidden()
                    .child(PackageIdentity::render_inline_glyph(pkg, 14.0, theme))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .text_ellipsis()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_sm()
                            .text_color(if is_selected {
                                theme.accent
                            } else {
                                theme.text_primary
                            })
                            .child(pkg.name.clone()),
                    ),
            )
            // Column 2: Version en police monospace
            .child(
                div()
                    .w(px(105.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_xs()
                    .font_family("monospace")
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_muted)
                    .child(if pkg.has_update {
                        format!("→ {}", pkg.new_version.as_deref().unwrap_or(&pkg.version))
                    } else {
                        pkg.version.clone()
                    }),
            )
            // Column 3: Source textuelle propre ("Arch / extra", sans badge chip)
            .child(
                div()
                    .w(px(105.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_xs()
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_secondary)
                    .child(source_display),
            )
            // Column 4: Size en police monospace aligné à droite avec padding
            .child(
                div()
                    .w(px(80.0))
                    .flex()
                    .justify_end()
                    .pr_3()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_xs()
                    .font_family("monospace")
                    .text_color(theme.text_secondary)
                    .child(Self::display_size(pkg)),
            )
            // Column 5: Status textuel desktop calme avec indicateur subtil (sans badge pill)
            .child(
                div()
                    .w(px(85.0))
                    .flex()
                    .items_center()
                    .child(if pkg.has_update {
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .text_xs()
                            .text_color(theme.warning_text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(div().size(px(5.0)).rounded_full().bg(theme.warning))
                            .child("Update")
                    } else if pkg.is_installed {
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .text_xs()
                            .text_color(theme.success_text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(div().size(px(5.0)).rounded_full().bg(theme.success))
                            .child("Installed")
                    } else {
                        div()
                            .text_xs()
                            .font_weight(FontWeight::NORMAL)
                            .text_color(theme.text_muted)
                            .child("Available")
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::models::{AlpmPackage, AppImageItem, AurPackage};
    use core::prelude::v1::test;

    #[test]
    fn test_format_bytes() {
        assert_eq!(PackageTable::format_bytes(500), "500 B");
        assert_eq!(PackageTable::format_bytes(2048), "2.0 KiB");
        assert_eq!(PackageTable::format_bytes(5 * 1024 * 1024), "5.0 MiB");
        assert_eq!(
            PackageTable::format_bytes(3 * 1024 * 1024 * 1024),
            "3.0 GiB"
        );
    }

    #[test]
    fn test_display_size_truthfulness() {
        // ALPM with installed size
        let alpm = UnifiedPackage {
            name: "alpm-pkg".into(),
            version: "1.0".into(),
            description: "".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "core".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage {
                installed_size: Some(10 * 1024 * 1024),
                ..Default::default()
            }),
        };
        assert_eq!(PackageTable::display_size(&alpm), "10.0 MiB");

        // AUR package has no size in search hit -> neutral em-dash, not 0 B
        let aur = UnifiedPackage {
            name: "aur-pkg".into(),
            version: "1.0".into(),
            description: "".into(),
            source_type: "AUR".into(),
            repository_or_remote: "aur".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Aur(AurPackage::default()),
        };
        assert_eq!(PackageTable::display_size(&aur), "—");

        // AppImage with size
        let appimage = UnifiedPackage {
            name: "appimage-pkg".into(),
            version: "1.0".into(),
            description: "".into(),
            source_type: "AppImage".into(),
            repository_or_remote: "appimage".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::AppImage(AppImageItem {
                size_on_disk: Some(50 * 1024 * 1024),
                ..Default::default()
            }),
        };
        assert_eq!(PackageTable::display_size(&appimage), "50.0 MiB");
    }
}
