use crate::backend::models::{UnifiedPackage, UnifiedPackageSource};
use crate::components::package_identity::PackageIdentity;
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use gpui::prelude::FluentBuilder;
use gpui::*;

/// The width of the measurement: the bar and its scale.
pub const MEASURE_WIDTH: f32 = 96.0;

pub struct PackageTable;

impl PackageTable {
    /// A measurement: the value drawn against a shared scale.
    ///
    /// The scale is the same for every row of the scope — the largest size
    /// in view — so the bars can be read against each other. The exact figure
    /// stays in its own column: the bar is the reading, the figure is the
    /// truth.
    pub fn render_measurement(
        bytes: Option<u64>,
        scale_max: u64,
        theme: &Theme,
    ) -> impl IntoElement {
        // The scale is logarithmic: the sizes span orders of magnitude, and a
        // linear scale would bury every small package under a sliver. The
        // header says so — a bar whose length is not a proportion must not
        // pretend to be one.
        let ratio = match (bytes, scale_max) {
            (Some(size), max) if max > 0 => {
                (1.0 + size as f32).ln() / (1.0 + max as f32).ln()
            }
            _ => 0.0,
        };
        let bar = theme.accent;
        div()
            .w(px(MEASURE_WIDTH))
            .h(px(6.0))
            .rounded_xs()
            .bg(theme.border)
            .relative()
            .child(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(0.0))
                    .h(px(6.0))
                    .w(px(MEASURE_WIDTH * ratio))
                    .rounded_xs()
                    .bg(Rgba {
                        a: 0.55,
                        ..bar
                    }),
            )
            .child(
                // The end-marker: where the reading stops.
                div()
                    .absolute()
                    .left(px((MEASURE_WIDTH * ratio - 1.0).max(0.0)))
                    .top(px(-3.0))
                    .w(px(2.0))
                    .h(px(12.0))
                    .bg(if bytes.is_some() { bar } else { theme.border }),
            )
    }

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

    /// The size of a package, in bytes — the value the measurement is drawn
    /// from.
    pub fn size_bytes(pkg: &UnifiedPackage) -> Option<u64> {
        let opt_size = match &pkg.inner {
            UnifiedPackageSource::Standard(alpm) => {
                alpm.installed_size.or(alpm.download_size).or(alpm.size)
            }
            UnifiedPackageSource::Flatpak(fp) => fp.installed_size.or(fp.download_size),
            UnifiedPackageSource::AppImage(ai) => ai.size_on_disk,
            UnifiedPackageSource::Aur(_) => None,
            UnifiedPackageSource::Cargo(_)
            | UnifiedPackageSource::Npm(_)
            | UnifiedPackageSource::Release(_) => None,
        };
        opt_size.filter(|size| *size > 0)
    }

    /// The largest size of a set: the top of the shared scale.
    pub fn scale_max(packages: &[UnifiedPackage]) -> u64 {
        packages
            .iter()
            .filter_map(Self::size_bytes)
            .max()
            .unwrap_or(0)
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

    /// The header of the readout. The measurement's column names the top of
    /// the scale: the bars are read against it.
    pub fn render_header(theme: &Theme, scale_max: u64) -> impl IntoElement {
        div()
            .h(px(UiMetrics::ROW_HEIGHT_HEADER))
            .w_full()
            .flex()
            .items_center()
            .px_3()
            .bg(gpui::rgba(0x00000000))
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
                    .w(px(MEASURE_WIDTH))
                    .flex()
                    .justify_end()
                    .pr_2()
                    .overflow_hidden()
                    .text_ellipsis()
                    .text_xs()
                    .font_family("monospace")
                    .font_weight(FontWeight::NORMAL)
                    .text_color(theme.text_muted)
                    .child(if scale_max > 0 {
                        format!("log 0 — {}", Self::format_bytes(scale_max))
                    } else {
                        "—".to_string()
                    }),
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
        scale_max: u64,
    ) -> impl IntoElement {
        // The rows are engraved into the recess: the ground is the recess's
        // own, and the rules are what separate them. Only the selected row
        // is lifted off it.
        let bg_color = if is_selected {
            theme.bg_surface_active
        } else {
            gpui::rgba(0x00000000)
        };

        // La sélection est marquée une fois : par la barre du porteur, que le
        // renfoncement pose sur la ligne. La règle de la ligne reste la règle
        // du tableau.
        let border_color = theme.border;

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
            // Column 4: the measurement — the value against the shared scale
            .child(
                div()
                    .w(px(MEASURE_WIDTH))
                    .flex()
                    .justify_end()
                    .pr_2()
                    .overflow_hidden()
                    .child(Self::render_measurement(
                        Self::size_bytes(pkg),
                        scale_max,
                        theme,
                    )),
            )
            // Column 5: Size en police monospace aligné à droite avec padding
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
