use crate::backend::models::UnifiedPackage;
use crate::components::package_identity::PackageIdentity;
use crate::icons::AppIcon;
use crate::state::{canonical_install_command, InspectorTab, PackageCapabilities};
use crate::theme::Theme;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub type TabSelectHandler = Rc<dyn Fn(InspectorTab, &mut Window, &mut App) + 'static>;
pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct InspectorHeaderProps<'a> {
    pub package: &'a UnifiedPackage,
    pub capabilities: &'a PackageCapabilities,
    pub active_tab: InspectorTab,
    pub theme: &'a Theme,
    pub is_busy: bool,
    pub copy_feedback: bool,
    pub on_select_tab: TabSelectHandler,
    pub on_install: Option<WindowActionHandler>,
    pub on_remove: Option<WindowActionHandler>,
    pub on_copy_install_cmd: Option<WindowActionHandler>,
}

pub struct InspectorHeader;

impl InspectorHeader {
    /// Formate la ligne de métadonnées calme du bureau (Arch · extra · ● Installed)
    pub fn format_metadata_line(pkg: &UnifiedPackage) -> (String, &'static str) {
        let source_name = match pkg.source_type.to_uppercase().as_str() {
            "ALPM" => "Arch",
            "AUR" => "AUR",
            "FLATPAK" => "Flatpak",
            "APPIMAGE" => "AppImage",
            _ => &pkg.source_type,
        };

        let mut meta = source_name.to_string();
        if !pkg.repository_or_remote.is_empty() {
            meta.push_str(" \u{00B7} ");
            meta.push_str(&pkg.repository_or_remote);
        }

        let status = if pkg.has_update {
            "Update available"
        } else if pkg.is_installed {
            "Installed"
        } else {
            "Available"
        };

        (meta, status)
    }

    fn render_tab_item(
        tab: InspectorTab,
        active_tab: InspectorTab,
        theme: &Theme,
        on_select: TabSelectHandler,
    ) -> impl IntoElement {
        let is_active = tab == active_tab;
        let hover_text = theme.text_primary;
        let element_id = match tab {
            InspectorTab::Overview => "inspector_tab_overview",
            InspectorTab::Dependencies => "inspector_tab_dependencies",
            InspectorTab::FilesBuild => "inspector_tab_files_build",
        };
        let focus_border = theme.border_focus;
        let on_select_key = on_select.clone();

        div()
            .id(element_id)
            .focusable()
            .tab_stop(true)
            .focus(move |s| s.border_1().border_color(focus_border))
            .on_key_down(move |event, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "enter" || key == "space" {
                    on_select_key(tab, window, cx);
                }
            })
            .flex()
            .items_center()
            .gap(px(4.0))
            .px_2()
            .py_1p5()
            .cursor_pointer()
            .text_xs()
            .font_weight(if is_active {
                FontWeight::BOLD
            } else {
                FontWeight::MEDIUM
            })
            .text_color(if is_active {
                theme.accent
            } else {
                theme.text_secondary
            })
            .border_b_2()
            .border_color(if is_active {
                theme.accent
            } else {
                rgba(0x00000000)
            })
            .when(!is_active, move |el| {
                el.hover(move |s| s.text_color(hover_text))
            })
            .child(
                svg()
                    .path(tab.icon().path())
                    .size_3_5()
                    .text_color(if is_active {
                        theme.accent
                    } else {
                        theme.text_secondary
                    }),
            )
            .child(tab.label())
            .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                on_select(tab, window, cx);
            })
    }

    pub fn render(props: InspectorHeaderProps) -> impl IntoElement {
        let theme = props.theme;
        let pkg = props.package;
        let caps = props.capabilities;
        let is_busy = props.is_busy;

        // 1. Identity avatar (32x32) + Compact Title & Calm Desktop Metadata
        let avatar = PackageIdentity::render_avatar(pkg, 32.0, 6.0, theme);

        let (meta_origin, status_label) = Self::format_metadata_line(pkg);
        let (status_color, status_dot) = if pkg.has_update {
            (theme.warning_text, theme.warning)
        } else if pkg.is_installed {
            (theme.success_text, theme.success)
        } else {
            (theme.text_muted, theme.border_focus)
        };

        let mut version_row = div().flex().items_center().gap_1().flex_shrink_0().child(
            div()
                .text_xs()
                .font_family("monospace")
                .text_color(theme.text_muted)
                .child(pkg.version.clone()),
        );

        if pkg.has_update {
            if let Some(ref nv) = pkg.new_version {
                version_row = version_row.child(
                    div()
                        .text_xs()
                        .font_family("monospace")
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.warning_text)
                        .child(format!(" → {}", nv)),
                );
            } else {
                version_row = version_row.child(
                    div()
                        .text_xs()
                        .font_family("monospace")
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.warning_text)
                        .child(" (update)"),
                );
            }
        }

        let title_column = div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .flex_1()
            .min_w_0()
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(pkg.name.clone()),
                    )
                    .child(version_row),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(div().child(meta_origin))
                    .child(div().child("\u{00B7}"))
                    .child(
                        div()
                            .size(px(6.0))
                            .rounded_full()
                            .bg(status_dot)
                            .flex_shrink_0(),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(status_color)
                            .child(status_label),
                    ),
            );

        let identity_row = div()
            .flex()
            .items_center()
            .gap_3()
            .w_full()
            .child(avatar)
            .child(title_column);

        // 2. Action buttons row (wrapping safely on tight viewports)
        let mut actions_row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .mt_2p5()
            .mb_2p5();

        if caps.can_remove {
            let danger_hover = theme.danger_hover;
            let focus_border = theme.border_focus;
            let on_remove_key = props.on_remove.clone();
            let mut remove_btn = div()
                .id("inspector_remove_btn")
                .px_3()
                .py_1()
                .rounded_md()
                .bg(if is_busy { theme.border } else { theme.danger })
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_busy {
                    theme.text_muted
                } else {
                    theme.bg_app
                });

            if !is_busy {
                remove_btn = remove_btn
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .cursor_pointer()
                    .hover(move |s| s.bg(danger_hover));
                if let Some(on_rm) = props.on_remove {
                    remove_btn = remove_btn
                        .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                            on_rm(window, cx);
                        })
                        .on_key_down(move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                if let Some(ref cb) = on_remove_key {
                                    cb(window, cx);
                                }
                            }
                        });
                }
            }

            actions_row = actions_row.child(remove_btn.child("Uninstall"));
        } else if caps.can_install {
            let accent_hover = theme.accent_hover;
            let focus_border = theme.border_focus;
            let on_install_key = props.on_install.clone();
            let mut install_btn = div()
                .id("inspector_install_btn")
                .px_3()
                .py_1()
                .rounded_md()
                .bg(if is_busy { theme.border } else { theme.accent })
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_busy {
                    theme.text_muted
                } else {
                    theme.bg_app
                });

            if !is_busy {
                install_btn = install_btn
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .cursor_pointer()
                    .hover(move |s| s.bg(accent_hover));
                if let Some(on_inst) = props.on_install {
                    install_btn = install_btn
                        .on_key_down(move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                if let Some(ref cb) = on_install_key {
                                    cb(window, cx);
                                }
                            }
                        })
                        .on_mouse_down(MouseButton::Left, move |_e, w, cx| on_inst(w, cx));
                }
            }
            actions_row = actions_row.child(install_btn.child("Install"));
        }

        // Copy install command button
        if canonical_install_command(pkg).is_some() {
            let on_copy = props.on_copy_install_cmd.clone();
            let on_copy_key = props.on_copy_install_cmd.clone();
            let focus_border = theme.border_focus;
            let copy_btn = div()
                .id("inspector_copy_cmd_btn")
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(focus_border))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        if let Some(ref cb) = on_copy_key {
                            cb(window, cx);
                        }
                    }
                })
                .flex()
                .items_center()
                .gap(px(4.0))
                .px_2p5()
                .py_1()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(if props.copy_feedback {
                    theme.success
                } else {
                    theme.border
                })
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(if props.copy_feedback {
                    theme.success_text
                } else {
                    theme.text_secondary
                })
                .cursor_pointer()
                .hover({
                    let h = theme.bg_surface_hover;
                    move |s| s.bg(h)
                })
                .child(
                    svg()
                        .path(if props.copy_feedback {
                            AppIcon::Check.path()
                        } else {
                            AppIcon::Copy.path()
                        })
                        .size_3_5()
                        .text_color(if props.copy_feedback {
                            theme.success
                        } else {
                            theme.text_secondary
                        }),
                )
                .child(if props.copy_feedback {
                    "Copied command!"
                } else {
                    "Copy install command"
                })
                .on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                    if let Some(ref cb) = on_copy {
                        cb(window, cx);
                    }
                });

            actions_row = actions_row.child(copy_btn);
        }

        // 3. Tab bar navigation
        let on_tab = props.on_select_tab.clone();
        let tab_bar = div()
            .flex()
            .items_center()
            .gap_1()
            .border_b_1()
            .border_color(theme.border)
            .child(Self::render_tab_item(
                InspectorTab::Overview,
                props.active_tab,
                theme,
                on_tab.clone(),
            ))
            .child(Self::render_tab_item(
                InspectorTab::Dependencies,
                props.active_tab,
                theme,
                on_tab.clone(),
            ))
            .child(Self::render_tab_item(
                InspectorTab::FilesBuild,
                props.active_tab,
                theme,
                on_tab,
            ));

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(identity_row)
            .child(actions_row)
            .child(tab_bar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::models::{AlpmPackage, UnifiedPackageSource};

    #[core::prelude::v1::test]
    fn test_inspector_header_metadata_line_formatting() {
        let pkg_installed = UnifiedPackage {
            name: "ripgrep".into(),
            version: "14.1.0-1".into(),
            description: "search".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let (meta, status) = InspectorHeader::format_metadata_line(&pkg_installed);
        assert_eq!(meta, "Arch \u{00B7} extra");
        assert_eq!(status, "Installed");

        let pkg_update = UnifiedPackage {
            name: "firefox".into(),
            version: "128.0".into(),
            description: "browser".into(),
            source_type: "AUR".into(),
            repository_or_remote: "".into(),
            is_installed: true,
            has_update: true,
            new_version: Some("129.0".into()),
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let (meta2, status2) = InspectorHeader::format_metadata_line(&pkg_update);
        assert_eq!(meta2, "AUR");
        assert_eq!(status2, "Update available");

        let pkg_flatpak = UnifiedPackage {
            name: "org.blender.Blender".into(),
            version: "4.2.0".into(),
            description: "3d".into(),
            source_type: "Flatpak".into(),
            repository_or_remote: "flathub".into(),
            is_installed: false,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let (meta3, status3) = InspectorHeader::format_metadata_line(&pkg_flatpak);
        assert_eq!(meta3, "Flatpak \u{00B7} flathub");
        assert_eq!(status3, "Available");
    }
}
