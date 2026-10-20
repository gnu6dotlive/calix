use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};
use crate::components::semantic_value::StringActionHandler;
use crate::state::{DependencyKind, DependencyRef, SemanticTarget};
use crate::theme::Theme;
use gpui::*;

pub struct DependenciesViewProps<'a> {
    pub package: &'a UnifiedPackage,
    pub alpm_details: Option<&'a AlpmPackage>,
    pub theme: &'a Theme,
    pub on_navigate_package: Option<StringActionHandler>,
}

pub struct DependenciesView;

impl DependenciesView {
    fn render_dep_pill(
        dep: DependencyRef,
        theme: &Theme,
        on_navigate: Option<StringActionHandler>,
    ) -> impl IntoElement {
        let hover_bg = theme.bg_surface_hover;
        let hover_border = theme.accent;
        let pkg_name = dep.name.clone();
        let is_nav = dep.is_navigable;

        let base = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(4.0))
            .px_2p5()
            .py_1()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .text_xs();

        let pill = if is_nav {
            base.cursor_pointer()
                .hover(move |s| s.bg(hover_bg).border_color(hover_border))
        } else {
            base
        };

        let mut content = pill.child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_color(if is_nav {
                    theme.accent
                } else {
                    theme.text_primary
                })
                .child(dep.name.clone()),
        );

        if let Some(constraint) = dep.constraint {
            content = content.child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.text_muted)
                    .child(constraint),
            );
        }

        if let Some(desc) = dep.description {
            content = content.child(
                div()
                    .text_color(theme.text_secondary)
                    .child(format!(": {}", desc)),
            );
        }

        if is_nav {
            let target = SemanticTarget::Package(pkg_name);
            content.on_mouse_down(MouseButton::Left, move |_ev, window, cx| {
                if let SemanticTarget::Package(ref name) = target {
                    if let Some(ref nav) = on_navigate {
                        nav(name.clone(), window, cx);
                    }
                }
            })
        } else {
            content
        }
    }

    fn render_section(
        kind: DependencyKind,
        raw_list: &[String],
        theme: &Theme,
        on_navigate: Option<StringActionHandler>,
    ) -> Option<impl IntoElement> {
        if raw_list.is_empty() {
            return None;
        }

        let pills = raw_list.iter().map(|raw| {
            let parsed = DependencyRef::parse(raw, kind);
            Self::render_dep_pill(parsed, theme, on_navigate.clone())
        });

        Some(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .mb_4()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_muted)
                        .child(format!(
                            "{} ({})",
                            kind.label().to_uppercase(),
                            raw_list.len()
                        )),
                )
                .child(div().flex().flex_wrap().gap_2().children(pills)),
        )
    }

    pub fn render(props: DependenciesViewProps) -> impl IntoElement {
        let theme = props.theme;
        let pkg = props.package;
        let alpm = props.alpm_details;
        let on_nav = props.on_navigate_package.clone();

        // Si source Flatpak ou AppImage, afficher explication de sandbox
        if pkg.source_type == "Flatpak" {
            return div()
                .flex()
                .flex_col()
                .p_6()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(theme.border)
                .gap_2()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .text_color(theme.text_primary)
                        .child("Sandboxed Flatpak Runtime"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text_secondary)
                        .line_height(relative(1.4))
                        .child("Flatpak applications bundle or link to shared ostree runtimes (e.g. org.freedesktop.Platform) and manage internal dependencies within an isolated sandbox environment."),
                );
        }

        if pkg.source_type == "AppImage" {
            return div()
                .flex()
                .flex_col()
                .p_6()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(theme.border)
                .gap_2()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .text_color(theme.text_primary)
                        .child("Self-Contained AppImage"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text_secondary)
                        .line_height(relative(1.4))
                        .child("AppImage binaries contain all necessary runtime dependencies and libraries embedded directly in the executable image."),
                );
        }

        // Extraction des listes de dépendances selon la source ALPM ou AUR
        let (depends, opt_depends, make_depends, conflicts, provides, required_by, optional_for) =
            match &pkg.inner {
                UnifiedPackageSource::Standard(a) => (
                    if a.depends.is_empty() {
                        alpm.map(|d| d.depends.clone()).unwrap_or_default()
                    } else {
                        a.depends.clone()
                    },
                    if a.opt_depends.is_empty() {
                        alpm.map(|d| d.opt_depends.clone()).unwrap_or_default()
                    } else {
                        a.opt_depends.clone()
                    },
                    Vec::new(),
                    if a.conflicts.is_empty() {
                        alpm.map(|d| d.conflicts.clone()).unwrap_or_default()
                    } else {
                        a.conflicts.clone()
                    },
                    if a.provides.is_empty() {
                        alpm.map(|d| d.provides.clone()).unwrap_or_default()
                    } else {
                        a.provides.clone()
                    },
                    alpm.map(|d| d.required_by.clone()).unwrap_or_default(),
                    alpm.map(|d| d.optional_for.clone()).unwrap_or_default(),
                ),
                UnifiedPackageSource::Aur(a) => (
                    a.depends.clone().unwrap_or_default(),
                    a.opt_depends.clone().unwrap_or_default(),
                    a.make_depends.clone().unwrap_or_default(),
                    a.conflicts.clone().unwrap_or_default(),
                    a.provides.clone().unwrap_or_default(),
                    Vec::new(),
                    Vec::new(),
                ),
                _ => (
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ),
            };

        let mut sections = Vec::new();

        if let Some(s) =
            Self::render_section(DependencyKind::Runtime, &depends, theme, on_nav.clone())
        {
            sections.push(s.into_any_element());
        }
        if let Some(s) = Self::render_section(
            DependencyKind::Optional,
            &opt_depends,
            theme,
            on_nav.clone(),
        ) {
            sections.push(s.into_any_element());
        }
        if let Some(s) =
            Self::render_section(DependencyKind::Make, &make_depends, theme, on_nav.clone())
        {
            sections.push(s.into_any_element());
        }
        if let Some(s) =
            Self::render_section(DependencyKind::Conflict, &conflicts, theme, on_nav.clone())
        {
            sections.push(s.into_any_element());
        }
        if let Some(s) =
            Self::render_section(DependencyKind::Provides, &provides, theme, on_nav.clone())
        {
            sections.push(s.into_any_element());
        }
        if let Some(s) = Self::render_section(
            DependencyKind::RequiredBy,
            &required_by,
            theme,
            on_nav.clone(),
        ) {
            sections.push(s.into_any_element());
        }
        if let Some(s) =
            Self::render_section(DependencyKind::OptionalFor, &optional_for, theme, on_nav)
        {
            sections.push(s.into_any_element());
        }

        if sections.is_empty() {
            return div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .p_8()
                .text_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(theme.text_muted)
                        .child("No declared dependencies for this package."),
                );
        }

        div().flex().flex_col().py_4().children(sections)
    }
}
