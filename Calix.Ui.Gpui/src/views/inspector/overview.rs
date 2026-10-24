use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};
use crate::components::package_table::PackageTable;
use crate::components::semantic_value::{SemanticValue, StringActionHandler};
use crate::state::{AlsoIn, SemanticTarget};
use crate::theme::Theme;
use gpui::*;

pub struct OverviewViewProps<'a> {
    pub package: &'a UnifiedPackage,
    pub alpm_details: Option<&'a AlpmPackage>,
    /// The other sources where the same name is found.
    pub also_in: &'a [AlsoIn],
    pub theme: &'a Theme,
    pub on_navigate_package: Option<StringActionHandler>,
}

pub struct OverviewView;

impl OverviewView {
    fn render_meta_item(
        label: &'static str,
        value: impl IntoElement,
        theme: &Theme,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .p_2()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text_muted)
                    .child(label),
            )
            .child(div().text_sm().text_color(theme.text_primary).child(value))
    }

    pub fn render(props: OverviewViewProps) -> impl IntoElement {
        let theme = props.theme;
        let pkg = props.package;
        let alpm = props.alpm_details;

        // Extraction des métadonnées
        let upstream_url = match &pkg.inner {
            UnifiedPackageSource::Standard(a) => a.url.clone(),
            UnifiedPackageSource::Aur(a) => a.url.clone(),
            UnifiedPackageSource::Flatpak(fp) => fp
                .id
                .as_ref()
                .map(|id| format!("https://flathub.org/apps/{}", id)),
            UnifiedPackageSource::AppImage(ai) => ai.update_url.clone(),
            UnifiedPackageSource::Cargo(_) => None,
            UnifiedPackageSource::Npm(_) => None,
            UnifiedPackageSource::Release(r) => r.repo.as_ref().map(|repo| {
                match r.provider.as_deref() {
                    Some("github") => format!("https://github.com/{}", repo),
                    Some("gitlab") => format!("https://gitlab.com/{}", repo),
                    _ => repo.clone(),
                }
            }),
        }
        .or_else(|| alpm.and_then(|a| a.url.clone()));

        let licenses = match &pkg.inner {
            UnifiedPackageSource::Standard(a) => {
                if a.licenses.is_empty() {
                    alpm.map(|d| d.licenses.join(", ")).unwrap_or_default()
                } else {
                    a.licenses.join(", ")
                }
            }
            UnifiedPackageSource::Aur(a) => {
                a.license.as_ref().map(|l| l.join(", ")).unwrap_or_default()
            }
            UnifiedPackageSource::Flatpak(_) => "Unknown / Flathub".to_string(),
            UnifiedPackageSource::AppImage(_) => "Unknown / Bundle".to_string(),
            UnifiedPackageSource::Cargo(_) => "Unknown / Crates.io".to_string(),
            UnifiedPackageSource::Npm(_) => "Unknown / npm".to_string(),
            UnifiedPackageSource::Release(_) => "Unknown / Release".to_string(),
        };

        let installed_size_str = match &pkg.inner {
            UnifiedPackageSource::Standard(a) => a
                .installed_size
                .or_else(|| alpm.and_then(|d| d.installed_size))
                .map(PackageTable::format_bytes),
            UnifiedPackageSource::Flatpak(fp) => fp.installed_size.map(PackageTable::format_bytes),
            UnifiedPackageSource::AppImage(ai) => ai.size_on_disk.map(PackageTable::format_bytes),
            UnifiedPackageSource::Aur(_) => None,
            UnifiedPackageSource::Cargo(_)
            | UnifiedPackageSource::Npm(_)
            | UnifiedPackageSource::Release(_) => None,
        };

        let download_size_str = match &pkg.inner {
            UnifiedPackageSource::Standard(a) => a
                .download_size
                .or_else(|| alpm.and_then(|d| d.download_size))
                .map(PackageTable::format_bytes),
            UnifiedPackageSource::Flatpak(fp) => fp.download_size.map(PackageTable::format_bytes),
            _ => None,
        };

        let maintainer = match &pkg.inner {
            UnifiedPackageSource::Aur(a) => a.maintainer.clone(),
            UnifiedPackageSource::Standard(_) => alpm.and_then(|d| d.package_base.clone()),
            _ => None,
        };

        let install_reason = alpm.and_then(|d| d.install_reason.clone());

        // 1. Description
        let desc_section = div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .mb_4()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child("DESCRIPTION"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.text_primary)
                    .line_height(relative(1.4))
                    .child(if pkg.description.is_empty() {
                        "No description available for this package.".to_string()
                    } else {
                        pkg.description.clone()
                    }),
            );

        // 2. Grille de métadonnées
        let mut meta_grid = div().flex().flex_col().gap_3();

        // Ligne 1 : Dépôt & Licence
        let row1 = div()
            .flex()
            .gap_3()
            .child(div().flex_1().child(Self::render_meta_item(
                "REPOSITORY / REMOTE",
                if pkg.repository_or_remote.is_empty() {
                    "—".to_string()
                } else {
                    pkg.repository_or_remote.clone()
                },
                theme,
            )))
            .child(div().flex_1().child(Self::render_meta_item(
                "LICENSE",
                if licenses.is_empty() {
                    "—".to_string()
                } else {
                    licenses
                },
                theme,
            )));
        meta_grid = meta_grid.child(row1);

        // Ligne 2 : Tailles (Installée & Téléchargement)
        let row2 = div()
            .flex()
            .gap_3()
            .child(div().flex_1().child(Self::render_meta_item(
                "INSTALLED SIZE",
                installed_size_str.unwrap_or_else(|| "—".to_string()),
                theme,
            )))
            .child(div().flex_1().child(Self::render_meta_item(
                "DOWNLOAD SIZE",
                download_size_str.unwrap_or_else(|| "—".to_string()),
                theme,
            )));
        meta_grid = meta_grid.child(row2);

        // Ligne 3 : URL amont & Mainteneur
        let on_nav = props.on_navigate_package.clone();
        let url_element: AnyElement = if let Some(url) = upstream_url {
            let url_c = url.clone();
            SemanticValue::render(url, SemanticTarget::ExternalUrl(url_c), theme, on_nav)
                .into_any_element()
        } else {
            div().child("—").into_any_element()
        };

        let maintainer_element: AnyElement = if let Some(m) = maintainer {
            let m_c = m.clone();
            SemanticValue::render(m, SemanticTarget::CopyText(m_c), theme, None).into_any_element()
        } else {
            div().child("—").into_any_element()
        };

        let row3 = div()
            .flex()
            .gap_3()
            .child(
                div()
                    .flex_1()
                    .child(Self::render_meta_item("UPSTREAM URL", url_element, theme)),
            )
            .child(div().flex_1().child(Self::render_meta_item(
                "MAINTAINER / BASE",
                maintainer_element,
                theme,
            )));
        meta_grid = meta_grid.child(row3);

        // Ligne 4 : Raison d'installation (si installée)
        if let Some(reason) = install_reason {
            meta_grid = meta_grid.child(Self::render_meta_item("INSTALL REASON", reason, theme));
        }

        // 3. Le même nom, ailleurs.
        //
        // Un nom n'est pas un paquet : `ripgrep` est un binaire dans les
        // dépôts officiels, une recette dans l'AUR, une crate sur crates.io.
        // La fiche le dit, et dit où le nom se trouve encore.
        let also_in_section = if props.also_in.is_empty() {
            div()
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .mt_4()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child("ALSO IN"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(props.also_in.iter().map(|entry| {
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.text_primary)
                                        .child(entry.source.short_label()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(crate::fonts::UI).font_features(crate::fonts::figures())
                                        .text_color(theme.text_muted)
                                        .child(entry.version.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child(if entry.installed {
                                            "installed".to_string()
                                        } else {
                                            entry.source.holds().to_string()
                                        }),
                                )
                        })),
                )
        };

        div()
            .flex()
            .flex_col()
            .py_4()
            .child(desc_section)
            .child(meta_grid)
            .child(also_in_section)
    }
}
