use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};
use crate::components::inspector_header::WindowActionHandler;
use crate::components::package_table::PackageTable;
use crate::components::semantic_value::SemanticValue;
use crate::icons::AppIcon;
use crate::state::SemanticTarget;
use crate::components::facts;
use crate::theme::Theme;
use gpui::*;

pub struct FilesBuildViewProps<'a> {
    pub package: &'a UnifiedPackage,
    pub alpm_details: Option<&'a AlpmPackage>,
    pub pkgbuild: Option<&'a String>,
    pub is_loading_pkgbuild: bool,
    pub theme: &'a Theme,
    pub on_copy_pkgbuild: Option<WindowActionHandler>,
}

pub struct FilesBuildView;

impl FilesBuildView {
    fn render_info_row(label: &'static str, value: impl IntoElement, theme: &Theme) -> Div {
        facts::fact(label, value, theme)
    }

    pub fn render(props: FilesBuildViewProps) -> impl IntoElement {
        let theme = props.theme;
        let pkg = props.package;
        let alpm = props.alpm_details;

        match &pkg.inner {
            UnifiedPackageSource::Aur(aur) => {
                let on_copy = props.on_copy_pkgbuild.clone();
                let pkgbuild_content = props.pkgbuild;

                let header_bar = div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .mb_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_muted)
                                    .child("AUR RECIPE (PKGBUILD)"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.accent_text)
                                    .child(format!("via calix search aur {} -p", pkg.name)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .px_2p5()
                            .py_1()
                            .rounded_md()
                            .bg(theme.bg_surface)
                            .border_1()
                            .border_color(theme.border)
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_secondary)
                            .cursor_pointer()
                            .hover({
                                let h = theme.bg_surface_hover;
                                move |s| s.bg(h)
                            })
                            .child(
                                svg()
                                    .path(AppIcon::Copy.path())
                                    .size_3_5()
                                    .text_color(theme.text_secondary),
                            )
                            .child("Copy PKGBUILD")
                            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                                if let Some(ref cb) = on_copy {
                                    cb(window, cx);
                                }
                            }),
                    );

                let meta_summary = facts::list([
                    facts::fact(
                        "Package base",
                        aur.package_base.clone().unwrap_or_else(|| pkg.name.clone()),
                        theme,
                    ),
                    facts::fact(
                        "Popularity",
                        format!(
                            "{:.2} · {} votes",
                            aur.popularity.unwrap_or(0.0),
                            aur.num_votes.unwrap_or(0)
                        ),
                        theme,
                    ),
                ])
                .mb_4();

                let viewer = if props.is_loading_pkgbuild {
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .p_12()
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.accent_text)
                                .child("Loading PKGBUILD recipe..."),
                        )
                        .into_any_element()
                } else if let Some(content) = pkgbuild_content {
                    div()
                        .p_3()
                        .rounded_md()
                        .bg(theme.bg_app)
                        .border_1()
                        .border_color(theme.border)
                        .text_xs()
                        .text_color(theme.text_primary)
                        .child(content.clone())
                        .into_any_element()
                } else {
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .p_12()
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.text_muted)
                                .child("PKGBUILD not yet fetched. Select this tab to load."),
                        )
                        .into_any_element()
                };

                div()
                    .flex()
                    .flex_col()
                    .py_4()
                    .child(header_bar)
                    .child(meta_summary)
                    .child(viewer)
            }

            UnifiedPackageSource::AppImage(ai) => {
                let size_str = ai
                    .size_on_disk
                    .map(PackageTable::format_bytes)
                    .unwrap_or_else(|| "—".to_string());
                let path_elem: AnyElement = if let Some(ref path) = ai.path {
                    let path_c = path.clone();
                    SemanticValue::render(
                        path.clone(),
                        SemanticTarget::FilePath(path_c),
                        theme,
                        None,
                    )
                    .into_any_element()
                } else {
                    div().child("—").into_any_element()
                };

                let update_elem: AnyElement = if let Some(ref url) = ai.update_url {
                    let url_c = url.clone();
                    SemanticValue::render(
                        url.clone(),
                        SemanticTarget::ExternalUrl(url_c),
                        theme,
                        None,
                    )
                    .into_any_element()
                } else {
                    div().child("—").into_any_element()
                };

                let info = div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .gap_2()
                    .child(Self::render_info_row("File", path_elem, theme))
                    .child(Self::render_info_row("Size on disk", size_str, theme))
                    .child(Self::render_info_row("Update URL", update_elem, theme))
                    .child(Self::render_info_row(
                        "Desktop entry",
                        ai.desktop_name.clone().unwrap_or_else(|| "—".to_string()),
                        theme,
                    ))
                    .child(Self::render_info_row(
                        "Icon name",
                        ai.icon_name.clone().unwrap_or_else(|| "—".to_string()),
                        theme,
                    ));

                div()
                    .flex()
                    .flex_col()
                    .py_4()
                    .gap_4()
                    .child(
                        facts::section_label("AppImage bundle", theme),
                    )
                    .child(info)
            }

            UnifiedPackageSource::Cargo(crate_) => {
                let binaries = if crate_.bins.is_empty() {
                    "—".to_string()
                } else {
                    crate_.bins.join(", ")
                };
                let info = div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .gap_2()
                    .child(Self::render_info_row("Crate", pkg.name.clone(), theme))
                    .child(Self::render_info_row("Version", crate_.version.clone(), theme))
                    .child(Self::render_info_row("Binaries", binaries, theme))
                    .child(Self::render_info_row(
                        "Registry",
                        if crate_.source.is_empty() {
                            "crates.io".to_string()
                        } else {
                            crate_.source.clone()
                        },
                        theme,
                    ));

                div()
                    .flex()
                    .flex_col()
                    .py_4()
                    .gap_4()
                    .child(
                        facts::section_label("Cargo crate", theme),
                    )
                    .child(info)
            }

            UnifiedPackageSource::Npm(package) => {
                let info = div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .gap_2()
                    .child(Self::render_info_row("Package", pkg.name.clone(), theme))
                    .child(Self::render_info_row("Version", package.version.clone(), theme))
                    .child(Self::render_info_row(
                        "Overridden",
                        if package.overridden { "yes" } else { "no" }.to_string(),
                        theme,
                    ));

                div()
                    .flex()
                    .flex_col()
                    .py_4()
                    .gap_4()
                    .child(
                        facts::section_label("npm package", theme),
                    )
                    .child(info)
            }

            UnifiedPackageSource::Release(release) => {
                let info = div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .gap_2()
                    .child(Self::render_info_row("Package", pkg.name.clone(), theme))
                    .child(Self::render_info_row(
                        "Installed",
                        release.installed_version.clone().unwrap_or_else(|| "—".into()),
                        theme,
                    ))
                    .child(Self::render_info_row(
                        "Latest",
                        release.latest_version.clone().unwrap_or_else(|| "—".into()),
                        theme,
                    ))
                    .child(Self::render_info_row(
                        "Upstream",
                        release.repo.clone().unwrap_or_else(|| "—".into()),
                        theme,
                    ))
                    .child(Self::render_info_row(
                        "Channel",
                        release.canal.clone().unwrap_or_else(|| "—".into()),
                        theme,
                    ));

                div()
                    .flex()
                    .flex_col()
                    .py_4()
                    .gap_4()
                    .child(
                        facts::section_label("Release", theme),
                    )
                    .child(info)
            }

            UnifiedPackageSource::Standard(_) => {
                let build_date = alpm
                    .and_then(|d| d.build_date.clone())
                    .unwrap_or_else(|| "—".to_string());
                let install_date = alpm
                    .and_then(|d| d.install_date.clone())
                    .unwrap_or_else(|| "—".to_string());
                let pkg_base = alpm
                    .and_then(|d| d.package_base.clone())
                    .unwrap_or_else(|| pkg.name.clone());

                let build_info = div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .gap_2()
                    .child(Self::render_info_row("Package base", pkg_base, theme))
                    .child(Self::render_info_row("Build date", build_date, theme))
                    .child(Self::render_info_row("Install date", install_date, theme))
                    .child(Self::render_info_row(
                        "Repository",
                        if pkg.repository_or_remote.is_empty() {
                            "—".to_string()
                        } else {
                            pkg.repository_or_remote.clone()
                        },
                        theme,
                    ));

                let notice = div()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .font_weight(FontWeight::BOLD)
                            .text_sm()
                            .text_color(theme.text_primary)
                            .mb_1()
                            .child(
                                svg()
                                    .path(AppIcon::Info.path())
                                    .size_3_5()
                                    .text_color(theme.accent_text),
                            )
                            .child("Detailed File Tree Listing"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .line_height(relative(1.4))
                            .child("Detailed package file tree listing is currently unexposed by the Calix CLI backend contract. Direct filesystem inspection via raw pacman is disabled to preserve frontend-backend boundary integrity."),
                    );

                div()
                    .flex()
                    .flex_col()
                    .py_4()
                    .gap_4()
                    .child(facts::section_label("Build and installation", theme))
                    .child(build_info)
                    .child(notice)
            }

            UnifiedPackageSource::Flatpak(fp) => {
                let app_id = fp
                    .app_id
                    .clone()
                    .or_else(|| fp.id.clone())
                    .unwrap_or_else(|| "—".to_string());
                let remote = fp.remote.clone().unwrap_or_else(|| "flathub".to_string());
                let app_type = fp
                    .app_type
                    .clone()
                    .unwrap_or_else(|| "desktop-application".to_string());

                let info = div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .gap_2()
                    .child(Self::render_info_row("Application ID", app_id, theme))
                    .child(Self::render_info_row("Remote catalog", remote, theme))
                    .child(Self::render_info_row("Application type", app_type, theme));

                let notice = div()
                    .p_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_sm()
                            .text_color(theme.text_primary)
                            .mb_1()
                            .child("Sandboxed Filesystem Hierarchy"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .line_height(relative(1.4))
                            .child("Flatpak sandboxed file hierarchies are managed directly by ostree and isolated per application sandbox under /var/lib/flatpak/app."),
                    );

                div()
                    .flex()
                    .flex_col()
                    .py_4()
                    .gap_4()
                    .child(
                        facts::section_label("Flatpak application", theme),
                    )
                    .child(info)
                    .child(notice)
            }
        }
    }
}
