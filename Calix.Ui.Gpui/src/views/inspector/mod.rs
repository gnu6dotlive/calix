pub mod dependencies;
pub mod files_build;
pub mod overview;

use crate::backend::models::{AlpmPackage, UnifiedPackage};
use crate::components::inspector_header::{
    InspectorHeader, InspectorHeaderProps, TabSelectHandler, WindowActionHandler,
};
use crate::components::semantic_value::StringActionHandler;
use crate::icons::AppIcon;
use crate::state::{InspectorTab, PackageCapabilities};
use crate::theme::Theme;
use crate::views::inspector::dependencies::{DependenciesView, DependenciesViewProps};
use crate::views::inspector::files_build::{FilesBuildView, FilesBuildViewProps};
use crate::views::inspector::overview::{OverviewView, OverviewViewProps};
use gpui::*;

pub struct PackageInspectorProps<'a> {
    pub package: Option<&'a UnifiedPackage>,
    pub alpm_details: Option<&'a AlpmPackage>,
    pub detail_error: Option<&'a str>,
    pub pkgbuild: Option<&'a String>,
    pub is_loading_pkgbuild: bool,
    pub active_tab: InspectorTab,
    pub theme: &'a Theme,
    pub is_busy: bool,
    pub copy_feedback: bool,
    pub on_select_tab: TabSelectHandler,
    pub on_install: Option<WindowActionHandler>,
    pub on_remove: Option<WindowActionHandler>,
    pub on_copy_install_cmd: Option<WindowActionHandler>,
    pub on_copy_pkgbuild: Option<WindowActionHandler>,
    pub on_navigate_package: Option<StringActionHandler>,
    pub on_retry_details: Option<WindowActionHandler>,
}

pub struct PackageInspectorView;

impl PackageInspectorView {
    pub fn render_with_motion(
        props: PackageInspectorProps,
        reduce_motion: bool,
        tab_epoch: u64,
    ) -> AnyElement {
        let theme = props.theme;

        let Some(pkg) = props.package else {
            return div()
                .id("empty_inspector")
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .size_full()
                .p_6()
                .bg(theme.bg_app)
                .child(
                    svg()
                        .path(AppIcon::PackageGeneric.path())
                        .size(px(40.0))
                        .text_color(theme.text_muted),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.text_primary)
                        .child("No Package Selected"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Select a package from the results to inspect its details, dependencies, and files."),
                )
                .into_any_element();
        };

        let capabilities = PackageCapabilities::derive(pkg, props.alpm_details);

        let header = InspectorHeader::render(InspectorHeaderProps {
            package: pkg,
            capabilities: &capabilities,
            active_tab: props.active_tab,
            theme,
            is_busy: props.is_busy,
            copy_feedback: props.copy_feedback,
            on_select_tab: props.on_select_tab.clone(),
            on_install: props.on_install.clone(),
            on_remove: props.on_remove.clone(),
            on_copy_install_cmd: props.on_copy_install_cmd.clone(),
        });

        let body: AnyElement = match props.active_tab {
            InspectorTab::Overview => OverviewView::render(OverviewViewProps {
                package: pkg,
                alpm_details: props.alpm_details,
                theme,
                on_navigate_package: props.on_navigate_package.clone(),
            })
            .into_any_element(),

            InspectorTab::Dependencies => DependenciesView::render(DependenciesViewProps {
                package: pkg,
                alpm_details: props.alpm_details,
                theme,
                on_navigate_package: props.on_navigate_package.clone(),
            })
            .into_any_element(),

            InspectorTab::FilesBuild => FilesBuildView::render(FilesBuildViewProps {
                package: pkg,
                alpm_details: props.alpm_details,
                pkgbuild: props.pkgbuild,
                is_loading_pkgbuild: props.is_loading_pkgbuild,
                theme,
                on_copy_pkgbuild: props.on_copy_pkgbuild.clone(),
            })
            .into_any_element(),
        };

        let animated_body = if reduce_motion {
            div()
                .id("inspector_tab_body")
                .child(body)
                .into_any_element()
        } else {
            div()
                .id("inspector_tab_body")
                .child(body)
                .with_animation(
                    ("tab-fade", tab_epoch as usize),
                    Animation::new(crate::state::MotionDurations::FAST)
                        .with_easing(gpui::ease_out_quint()),
                    |elem, delta| elem.opacity(delta),
                )
                .into_any_element()
        };

        let error_banner = if let Some(err) = props.detail_error {
            let on_retry = props.on_retry_details.clone();
            Some(
                div()
                    .id("detail_error_banner")
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .mb_4()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.danger)
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.danger)
                            .child(format!("Failed to load complete package details: {err}")),
                    )
                    .children(on_retry.map(|retry_cb| {
                        div()
                            .id("retry_details_btn")
                            .px_2p5()
                            .py_1()
                            .rounded_sm()
                            .bg(theme.accent)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.bg_app)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.accent_hover))
                            .child("Retry")
                            .on_mouse_down(MouseButton::Left, move |_ev, w, cx| {
                                retry_cb(w, cx);
                            })
                    })),
            )
        } else {
            None
        };

        div()
            .id("inspector_container")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .child(
                div()
                    .id("inspector_pinned_header")
                    .flex()
                    .flex_col()
                    .w_full()
                    .px_4()
                    .pt_3()
                    .bg(theme.bg_app)
                    .child(header),
            )
            .child(
                div()
                    .id("inspector_scroll_body")
                    .flex_1()
                    .overflow_scroll()
                    .px_4()
                    .py_3()
                    .children(error_banner)
                    .child(animated_body),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::models::{AlpmPackage, UnifiedPackageSource};

    #[core::prelude::v1::test]
    fn test_inspector_empty_state_and_populated_render() {
        let theme = Theme::dark();
        let on_tab: crate::components::inspector_header::TabSelectHandler =
            std::rc::Rc::new(|_, _, _| {});

        // 1. Empty state
        let empty_props = PackageInspectorProps {
            package: None,
            alpm_details: None,
            detail_error: None,
            pkgbuild: None,
            is_loading_pkgbuild: false,
            active_tab: InspectorTab::Overview,
            theme: &theme,
            is_busy: false,
            copy_feedback: false,
            on_select_tab: on_tab.clone(),
            on_install: None,
            on_remove: None,
            on_copy_install_cmd: None,
            on_copy_pkgbuild: None,
            on_navigate_package: None,
            on_retry_details: None,
        };
        let _empty_el = PackageInspectorView::render_with_motion(empty_props, true, 0);

        // 2. Populated package with reduced motion = false
        let pkg = UnifiedPackage {
            name: "ripgrep".into(),
            version: "14.1.0-1".into(),
            description: "fast search".into(),
            source_type: "ALPM".into(),
            repository_or_remote: "extra".into(),
            is_installed: true,
            has_update: false,
            new_version: None,
            inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
        };

        let populated_props = PackageInspectorProps {
            package: Some(&pkg),
            alpm_details: None,
            detail_error: Some("Simulated detail error"),
            pkgbuild: None,
            is_loading_pkgbuild: false,
            active_tab: InspectorTab::Overview,
            theme: &theme,
            is_busy: false,
            copy_feedback: false,
            on_select_tab: on_tab,
            on_install: None,
            on_remove: None,
            on_copy_install_cmd: None,
            on_copy_pkgbuild: None,
            on_navigate_package: None,
            on_retry_details: Some(std::rc::Rc::new(|_, _| {})),
        };
        let _pop_el = PackageInspectorView::render_with_motion(populated_props, false, 1);
    }
}
