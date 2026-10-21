use crate::backend::models::{UnifiedPackage, UnifiedPackageSource};
use crate::components::package_identity::PackageIdentity;
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

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

    /// La ligne du catalogue, telle que le catalogue la rend.
    ///
    /// C'est la ligne de la table — la même, au pixel près — avec la hauteur
    /// du rang et le geste qui la sélectionne. Le banc d'essai la rend
    /// directement : ce qui y est mesuré est ce que l'application dessine,
    /// et non une approximation faite pour le banc.
    pub fn render_selectable_row(
        pkg: &UnifiedPackage,
        is_selected: bool,
        theme: &Theme,
        compact: bool,
        scale_max: u64,
        on_select: Rc<dyn Fn(&mut Window, &mut App) + 'static>,
    ) -> impl IntoElement {
        let row_height = if compact {
            UiMetrics::ROW_HEIGHT_COMPACT
        } else {
            UiMetrics::ROW_HEIGHT_NORMAL
        };
        div()
            .id("package_row")
            // The handle the bench measures: the harness reads the bounds of
            // the row under this name. A noop outside the tests.
            .debug_selector(|| "package_row".to_string())
            .h(px(row_height))
            .child(Self::render_row(pkg, is_selected, theme, compact, scale_max))
            .on_mouse_down(MouseButton::Left, move |_e, w, cx| on_select(w, cx))
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
                    .id("package_row_name_cell")
                    // The handle the bench measures.
                    .debug_selector(|| "package_row_name_cell".to_string())
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .items_center()
                    .gap_2()
                    .overflow_hidden()
                    .child(PackageIdentity::render_inline_glyph(pkg, 14.0, theme))
                    .child(
                        div()
                            .id("package_row_name")
                    // The handle the bench measures.
                    .debug_selector(|| "package_row_name".to_string())
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
                    .id("package_row_version")
                    // The handle the bench measures.
                    .debug_selector(|| "package_row_version".to_string())
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
                    .id("package_row_source")
                    // The handle the bench measures.
                    .debug_selector(|| "package_row_source".to_string())
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
                    .id("package_row_measurement")
                    // The handle the bench measures.
                    .debug_selector(|| "package_row_measurement".to_string())
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
                    .id("package_row_size")
                    // The handle the bench measures.
                    .debug_selector(|| "package_row_size".to_string())
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
                    .id("package_row_status")
                    // The handle the bench measures.
                    .debug_selector(|| "package_row_status".to_string())
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
    use crate::backend::models::UnifiedPackage;
    use core::prelude::v1::test;
    use gpui::{div, point, px, size, Context, Render, Window};

    /// Les noms réels, échantillonnés dans l'inventaire de la machine de
    /// mesure (1884 paquets installés) : la médiane, le p80, le p90, et les
    /// extrêmes.
    ///
    /// Le critère du minimum du catalogue se tire de cette distribution : la
    /// largeur à laquelle le nom est entier pour 90 % des paquets réels.
    const REAL_NAMES: &[&str] = &[
        // the median and the short tail
        "7zip",
        "bc",
        "zotero",
        "alsa-utils",
        "intel-ucode",
        "libxkbcommon",
        // the p80
        "aws-sdk-cpp-s3",
        "vlc-plugin-alsa",
        // the p90
        "ruby-power_assert",
        "python-jaraco.classes",
        // the extreme
        "linux-cachyos-bore-lto-nvidia-open",
        "networkmanager-vpn-plugin-openvpn",
        "intel-oneapi-compiler-shared-runtime-libs",
        "intel-oneapi-compiler-dpcpp-cpp-runtime-libs",
    ];

    /// La ligne du catalogue, mesurée : ce que la largeur garantit, et ce
    /// qu'elle coûte quand elle diminue.
    ///
    /// C'est la **vraie** ligne — le composant que le catalogue rend, pas
    /// une approximation faite pour le banc. Elle est dessinée dans un
    /// conteneur de la largeur qu'on veut qualifier, et on lit les bornes que
    /// la fenêtre a dessinées : la part du nom, et les colonnes fixes.
    ///
    /// Ce que la mesure dit, et ce qu'elle ne dit pas :
    ///
    /// - elle dit ce que la ligne **tient** : le prix de ses colonnes fixes,
    ///   la part du nom à chaque largeur, et la largeur à laquelle le nom
    ///   cesse d'être coupé ;
    /// - elle ne dit pas que la `uniform_list` peint ses éléments : la liste
    ///   reste à qualifier dans l'application (audit, §4.7).
    #[gpui::test]
    fn the_row_is_measured_at_and_around_the_catalogue_minimum(
        cx: &mut gpui::TestAppContext,
    ) {
        struct Fixture {
            theme: Theme,
            pkg: UnifiedPackage,
            width: f32,
            unconstrained: bool,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                if self.unconstrained {
                    // The name, without the constraint that clips it: its
                    // natural width is a property of the text, not of the row.
                    // A flex row's child that does not grow takes its content's
                    // width — that is the natural width.
                    return div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .child(
                            div()
                                .id("package_row_name")
                                // The handle the bench measures.
                                .debug_selector(|| "package_row_name".to_string())
                                .flex_none()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(self.pkg.name.clone()),
                        )
                        .into_any_element();
                }
                div()
                    .w(px(self.width))
                    .child(PackageTable::render_selectable_row(
                        &self.pkg,
                        false,
                        &theme,
                        false,
                        u64::MAX,
                        std::rc::Rc::new(|_w, _cx| {}),
                    ))
                    .into_any_element()
            }
        }

        let cx = cx.add_empty_window();
        let theme = Theme::dark();
        let pkg = |name: &str| UnifiedPackage {
            name: name.into(),
            version: "14.1.0-1".into(),
            description: "a package".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let draw = |cx: &mut gpui::VisualTestContext, pkg: UnifiedPackage, width: f32| {
            let fixture = cx.new(|_| Fixture {
                theme: theme.clone(),
                pkg,
                width,
                unconstrained: false,
            });
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(width.max(200.0)), px(200.0)),
                |_window, _cx| fixture.clone(),
            );
        };
        let draw_natural = |cx: &mut gpui::VisualTestContext, pkg: UnifiedPackage| {
            let fixture = cx.new(|_| Fixture {
                theme: theme.clone(),
                pkg,
                width: 0.0,
                unconstrained: true,
            });
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(2000.0), px(200.0)),
                |_window, _cx| fixture.clone(),
            );
        };

        // The fixed columns: what the row spends before the name. They do
        // not move — they are the price of the name.
        draw(cx, pkg("ripgrep"), 800.0);
        let mut fixed = 0.0_f32;
        for (id, expected) in [
            ("package_row_version", 105.0_f32),
            ("package_row_source", 105.0),
            ("package_row_measurement", 96.0),
            ("package_row_size", 80.0),
            ("package_row_status", 85.0),
        ] {
            let bounds = cx
                .debug_bounds(id)
                .unwrap_or_else(|| panic!("{id} is drawn"));
            assert_eq!(bounds.size.width, px(expected), "{id} keeps its width");
            fixed += expected;
        }
        // And the row's own padding: twelve on each side.
        let row_floor = fixed + 24.0;
        assert_eq!(fixed, 471.0, "the fixed columns' total, measured");
        assert_eq!(row_floor, 495.0, "the row's floor, measured");

        // The names' natural widths: the text, uncut.
        //
        // The names are the *real* ones — the inventory of the machine this
        // was measured on (1884 packages), sampled across its distribution
        // and its extreme. The criterion for the catalogue's minimum is
        // drawn from that distribution, not from a name one picks.
        let mut natural = 0.0_f32;
        let mut widths: Vec<(f32, &'static str)> = Vec::new();
        for name in REAL_NAMES {
            draw_natural(cx, pkg(name));
            let width: f32 = cx
                .debug_bounds("package_row_name")
                .expect("the name is drawn")
                .size
                .width
                .into();
            widths.push((width, name));
            natural = natural.max(width);
        }
        for (width, name) in &widths {
            eprintln!("PROBE natural {name}: {width}");
        }

        // The name's share, at every width that matters.
        let mut shares: Vec<(f32, f32)> = Vec::new();
        for width in [400.0_f32, 480.0, 495.0, 520.0, 560.0, 600.0, 640.0, 720.0, 800.0] {
            draw(cx, pkg("browsers-neo-bin"), width);
            let cell: f32 = cx
                .debug_bounds("package_row_name_cell")
                .expect("the name's cell is drawn")
                .size
                .width
                .into();
            let name: f32 = cx
                .debug_bounds("package_row_name")
                .expect("the name is drawn")
                .size
                .width
                .into();
            if width >= row_floor {
                // The name takes what the fixed columns leave.
                assert!(
                    (cell - (width - row_floor)).abs() < 1.0,
                    "at {width} px, the name's cell is {cell} and the row leaves {}",
                    width - row_floor
                );
                assert!(
                    name <= cell + 0.5,
                    "at {width} px, the name is cut to its cell: {name} in {cell}"
                );
            } else {
                // Below the floor, the name is gone and the row's right end
                // is cut: the row overflows.
                assert_eq!(cell, 0.0, "at {width} px the name has no place");
            }
            eprintln!("PROBE at {width}: cell={cell} name={name}");
            shares.push((width, cell));
        }

        // ── The qualification ────────────────────────────────────────────
        //
        // The name's whole width: the row's floor, plus the identity glyph
        // and its gap, plus the name itself.
        let glyph_and_gap = 22.0;
        let whole = |name_width: f32| row_floor + glyph_and_gap + name_width;

        // The p90 of the real names, taken from the measured sample.
        let p80 = widths
            .iter()
            .find(|(width, _)| *width > 120.0)
            .map(|(width, _)| *width)
            .expect("the sample holds a p80");
        let p90 = widths
            .iter()
            .find(|(width, _)| *width > 140.0)
            .map(|(width, _)| *width)
            .expect("the sample holds a p90");
        eprintln!(
            "PROBE qualification: floor={row_floor} p80={p80} p90={p90}              whole_p80={} whole_p90={} longest={natural}",
            whole(p80),
            whole(p90)
        );

        // The catalogue's qualified minimum: the width at which the name is
        // whole for 90 % of the real names.
        let qualified = whole(p90);
        assert!(
            qualified > 560.0,
            "560 px ne suffit pas : le nom du p90 ({p90} px) y est coupé, il lui faut {qualified}"
        );

        // What 560 px buys, measured: the name is cut to a few characters.
        let at_minimum = shares
            .iter()
            .find(|(width, _)| *width == 560.0)
            .expect("measured at 560");
        assert_eq!(at_minimum.1, 65.0, "at 560 px the name's cell is 65 px");
        assert!(
            at_minimum.1 - glyph_and_gap < 60.0,
            "at 560 px, the name shows {} px of text — six characters",
            at_minimum.1 - glyph_and_gap
        );

        // The concession that would make 560 hold: the measurement bar is
        // the *reading* of the size, and the size has its own column — the
        // bar is what the row can do without. Its width is measured, so
        // what it frees is known, not guessed.
        let without_the_bar = row_floor - MEASURE_WIDTH;
        let name_at_560_without_the_bar = 560.0 - 24.0 - (fixed - MEASURE_WIDTH);
        eprintln!(
            "PROBE concession: without the bar the row's floor is {without_the_bar}              and the name at 560 has {name_at_560_without_the_bar} px"
        );
        assert!(
            name_at_560_without_the_bar >= p90,
            "sans la barre de mesure, le nom du p90 tiendrait à 560 px : {name_at_560_without_the_bar} pour {p90}"
        );
    }

    /// Le catalogue, qualification : la liste paints ses lignes, et ce
    /// qu'elles contiennent à chaque largeur.
    ///
    /// La qualification de la ligne (§ plus haut) portait sur la ligne
    /// *seule*. Celle-ci porte sur la **liste** : le vrai `uniform_list`, la
    /// vraie ligne, et les bornes que la fenêtre a dessinées.
    ///
    /// Une leçon du banc, gardée ici : une `uniform_list` dont la hauteur
    /// n'est pas donnée est posée à **zéro** par le chemin de la mise en
    /// page mesurée, et ne dessine alors aucune ligne. La liste du
    /// catalogue reçoit sa hauteur du flex (`flex_1`), que le banc ne
    /// résout pas ; le banc lui donne donc une hauteur certaine. Ce n'est
    /// pas une correction du produit : c'est la condition de lecture du
    /// banc.
    #[gpui::test]
    fn the_catalogue_list_draws_its_rows_at_every_width(cx: &mut gpui::TestAppContext) {
        struct Fixture {
            theme: Theme,
            packages: Vec<UnifiedPackage>,
            width: f32,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                let packages = self.packages.clone();
                let width = self.width;
                div()
                    .size_full()
                    .child(
                        div()
                            .w(px(width))
                            .h(px(300.0))
                            .child(
                                uniform_list("catalogue-list", packages.len(), move |range, _w, _cx| {
                                    range
                                        .map(|idx| {
                                            let pkg = &packages[idx];
                                            PackageTable::render_selectable_row(
                                                pkg,
                                                false,
                                                &theme,
                                                false,
                                                u64::MAX,
                                                std::rc::Rc::new(|_w, _cx| {}),
                                            )
                                            .into_any_element()
                                        })
                                        .collect::<Vec<_>>()
                                })
                                .h(px(300.0)),
                            ),
                    )
                    .into_any_element()
            }
        }

        let cx = cx.add_empty_window();
        let theme = Theme::dark();
        let make = |name: &str| UnifiedPackage {
            name: name.into(),
            version: "14.1.0-1".into(),
            description: "a package".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };
        let packages = vec![
            make("7zip"),
            make("browsers-neo-bin"),
            make("ruby-power_assert"),
        ];

        let mut drawn: Vec<(f32, f32, f32)> = Vec::new();
        for width in [400.0_f32, 480.0, 495.0, 520.0, 560.0, 660.0, 720.0, 800.0] {
            let fixture = cx.new(|_| Fixture {
                theme: theme.clone(),
                packages: packages.clone(),
                width,
            });
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(width.max(200.0)), px(400.0)),
                |_window, _cx| fixture.clone(),
            );

            // The row the list drew: its own bounds.
            let row = cx
                .debug_bounds("package_row")
                .expect("the list draws its rows");
            let row_width: f32 = row.size.width.into();
            let row_height: f32 = row.size.height.into();

            // The name's cell, in the row the list drew.
            let cell: f32 = cx
                .debug_bounds("package_row_name_cell")
                .map(|bounds| bounds.size.width.into())
                .unwrap_or(0.0);

            eprintln!("PROBE list at {width}: row {row_width}x{row_height}, name cell {cell}");
            drawn.push((width, row_width, cell));

            // The row is the real row: its height is the catalogue's.
            assert_eq!(
                row_height,
                crate::ui_metrics::UiMetrics::ROW_HEIGHT_NORMAL,
                "the list draws the real row, at its real height"
            );
        }

        // The row takes the list's width, and the name's cell is what the
        // fixed columns leave — the same rule as the row alone, now proved
        // *in the list*.
        for (width, row_width, cell) in &drawn {
            assert_eq!(
                *row_width, *width,
                "the row takes the list's width: {row_width} in {width}"
            );
            let expected = (*width - 495.0).max(0.0);
            assert!(
                (cell - expected).abs() < 1.0,
                "at {width} px, the name's cell is {cell} and the fixed columns leave {expected}"
            );
        }
    }

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
