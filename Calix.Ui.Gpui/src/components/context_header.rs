use crate::state::package_store::{SourceHealthMap, SourceHealthStatus};
use crate::state::query::{PackageStateFilter, SourceScope};
use crate::state::{NavScope, PackageSourceKind, Place};
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type ScopeSelectHandler = Rc<dyn Fn(NavScope, &mut Window, &mut App) + 'static>;
pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type SourceRetryHandler = Rc<dyn Fn(PackageSourceKind, &mut Window, &mut App) + 'static>;

pub struct ContextHeaderProps<'a> {
    /// Le lieu : la surface des paquets, Actualités ou Réglages.
    pub place: Place,
    /// Le périmètre courant, quand le lieu en a un.
    pub scope: NavScope,
    /// Le nombre de lignes que montre le périmètre.
    pub row_count: usize,
    /// Le nombre de lignes est inconnu (chargement en cours).
    pub row_count_unknown: bool,
    /// Les filtres en force, pour le résumé.
    pub source_scope: SourceScope,
    pub state_filter: PackageStateFilter,
    pub aur_enabled: bool,
    pub flatpak_enabled: bool,
    pub appimage_enabled: bool,
    /// Quand la réponse du périmètre a été vraie, quand elle est connue.
    pub age: Option<String>,
    /// La santé des sources : une source qui n'a pas répondu se dit.
    pub source_health: &'a SourceHealthMap,
    pub on_retry_source: SourceRetryHandler,
    pub on_select_scope: ScopeSelectHandler,
    pub on_clear_filters: WindowActionHandler,
    pub theme: &'a Theme,
}

/// Le résumé des filtres en force, quand ils ne sont pas à leur valeur par
/// défaut. `None` quand tout est par défaut : l'en-tête n'a alors rien à dire.
pub fn format_filter_summary(
    source_scope: &SourceScope,
    state_filter: PackageStateFilter,
    aur_enabled: bool,
    flatpak_enabled: bool,
    appimage_enabled: bool,
) -> Option<String> {
    let is_all_sources =
        source_scope.is_all_enabled(aur_enabled, flatpak_enabled, appimage_enabled);
    let is_all_states = state_filter == PackageStateFilter::All;
    if is_all_sources && is_all_states {
        return None;
    }

    let mut parts = Vec::new();
    if !is_all_sources {
        let mut names = Vec::new();
        if source_scope.alpm {
            names.push("Official");
        }
        if source_scope.aur && aur_enabled {
            names.push("AUR");
        }
        if source_scope.flatpak && flatpak_enabled {
            names.push("Flatpak");
        }
        if source_scope.appimage && appimage_enabled {
            names.push("AppImage");
        }
        if !names.is_empty() {
            parts.push(names.join(" + "));
        }
    }
    if !is_all_states {
        parts.push(state_filter.label().to_string());
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

pub struct ContextHeader;

impl ContextHeader {
    pub fn render(props: ContextHeaderProps) -> impl IntoElement {
        let theme = props.theme;
        let is_packages = props.place == Place::Packages;

        // ── Le titre : le périmètre, ou le lieu ─────────────────────────────
        let title = if is_packages {
            props.scope.label()
        } else {
            props.place.label()
        };

        // ── Le commutateur de périmètre ─────────────────────────────────────
        let mut switcher = div().flex().items_center().gap(px(2.0));
        for target in NavScope::ALL {
            let is_active = is_packages && *target == props.scope;
            let select_cb = props.on_select_scope.clone();
            let select_cb_key = select_cb.clone();
            let focus_border = theme.border_focus;
            let target_val = *target;
            let label = target.label();
            let icon = target.icon();

            let segment = div()
                .id(ElementId::Name(
                    format!("context_header_scope_{}", target.as_str()).into(),
                ))
                .flex()
                .items_center()
                .gap(px(6.0))
                .px(px(8.0))
                .py(px(2.0))
                .rounded_sm()
                .cursor_pointer()
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(focus_border))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        select_cb_key(target_val, window, cx);
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    select_cb(target_val, window, cx);
                })
                .bg(if is_active {
                    theme.bg_surface_active
                } else {
                    theme.bg_app
                })
                .child(
                    svg()
                        .path(icon.path())
                        .size_4()
                        .text_color(if is_active {
                            theme.accent
                        } else {
                            theme.text_secondary
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(if is_active {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(if is_active {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        })
                        .child(label),
                );

            switcher = switcher.child(segment);
        }

        // ── Le compte ───────────────────────────────────────────────────────
        let count = if props.row_count_unknown {
            "—".to_string()
        } else if props.row_count == 1 {
            "1 package".to_string()
        } else {
            format!("{} packages", props.row_count)
        };

        // ── La couverture : les écosystèmes du périmètre ────────────────────
        let coverage = if is_packages {
            // Les sources interrogeables, puis celles que le périmètre liste
            // sans pouvoir les interroger : la ligne le dit, pour que la
            // couverture ne mente pas sur ce qui se cherche.
            let inventory_only = props.scope.inventory_only_sources();
            let searchable: Vec<String> = props
                .scope
                .sources()
                .iter()
                .filter(|source| !inventory_only.contains(source))
                .map(|source| source.to_string())
                .collect();
            let listed: Vec<String> = inventory_only
                .iter()
                .map(|source| source.to_string())
                .collect();
            let mut parts = vec![searchable.join(" · ")];
            if !listed.is_empty() {
                parts.push(format!("{} (inventory only)", listed.join(" · ")));
            }
            parts.join("  +  ")
        } else {
            String::new()
        };

        // Une source qui n'a pas répondu se dit, et se réessaie : la barre
        // d'état a laissé la place, la santé reste.
        let failed_source = [
            PackageSourceKind::Alpm,
            PackageSourceKind::Aur,
            PackageSourceKind::Flatpak,
            PackageSourceKind::AppImage,
        ]
        .into_iter()
        .find(|kind| {
            let enabled = match kind {
                PackageSourceKind::Alpm => true,
                PackageSourceKind::Aur => props.aur_enabled,
                PackageSourceKind::Flatpak => props.flatpak_enabled,
                PackageSourceKind::AppImage => props.appimage_enabled,
                _ => false,
            };
            enabled
                && props
                    .source_health
                    .get(kind)
                    .is_some_and(|health| health.status == SourceHealthStatus::Failed)
        });

        let filter_summary = format_filter_summary(
            &props.source_scope,
            props.state_filter,
            props.aur_enabled,
            props.flatpak_enabled,
            props.appimage_enabled,
        );

        let on_clear = props.on_clear_filters.clone();
        let on_clear_key = on_clear.clone();
        let clear_action = filter_summary.as_ref().map(|summary| {
            let focus_border = theme.border_focus;
            div()
                .id("context_header_clear_filters")
                .flex()
                .items_center()
                .gap(px(4.0))
                .px(px(6.0))
                .py(px(2.0))
                .rounded_sm()
                .cursor_pointer()
                .focusable()
                .tab_stop(true)
                .focus(move |s| s.border_1().border_color(focus_border))
                .on_key_down(move |event, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if key == "enter" || key == "space" {
                        on_clear_key(window, cx);
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                    on_clear(window, cx);
                })
                .text_xs()
                .text_color(theme.accent)
                .hover(|s| s.bg(theme.bg_surface_hover).text_color(theme.accent_hover))
                .child(summary.clone())
        });

        // The second line has a height of its own: the text needs its line
        // box, and a line that is measured by its glyphs is a line that gets
        // cut.
        let mut second_line = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .overflow_hidden();
        if !coverage.is_empty() {
            second_line = second_line.child(
                div()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .truncate()
                    .child(coverage),
            );
        }
        if let Some(age) = props.age.as_ref() {
            second_line = second_line.child(
                div().text_xs().text_color(theme.text_muted).child(age.clone()),
            );
        }
        if let Some(clear) = clear_action {
            second_line = second_line.child(clear);
        }
        if let Some(failed) = failed_source {
            let on_retry = props.on_retry_source.clone();
            let kind = failed;
            second_line = second_line.child(
                div()
                    .id("context_header_retry_source")
                    .px(px(6.0))
                    .py(px(2.0))
                    .rounded_sm()
                    .cursor_pointer()
                    .text_xs()
                    .text_color(theme.danger)
                    .hover(|s| s.bg(theme.bg_surface_hover))
                    .child(format!("{} unavailable — retry", kind))
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        on_retry(kind, window, cx);
                    }),
            );
        }

        div()
            .id("context_header")
            // L'en-tête ne se comprime pas : la liste prend le reste. Sa
            // hauteur est celle de ses deux lignes — la colonne lui laisse
            // la sienne, et le contenu de l'en-tête ne se fait pas couper.
            .flex_shrink_0()
            .w_full()
            .flex()
            .flex_col()
            .justify_start()
            .px(px(16.0))
            .pt(px(2.0))
            .pb(px(4.0))
            .bg(theme.bg_sidebar)
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.text_primary)
                                    .child(title),
                            )
                            .child(switcher),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary)
                            .child(count),
                    ),
            )
            .child(second_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_filter_summary_is_silent_when_everything_is_by_default() {
        let scope = SourceScope::all_enabled(true, true, true);
        assert_eq!(
            format_filter_summary(&scope, PackageStateFilter::All, true, true, true),
            None
        );
    }

    #[test]
    fn the_filter_summary_names_the_filters_in_force() {
        let mut scope = SourceScope::all_enabled(true, true, true);
        scope.alpm = false;
        scope.flatpak = false;
        scope.appimage = false;
        assert_eq!(
            format_filter_summary(&scope, PackageStateFilter::All, true, true, true),
            Some("AUR".to_string())
        );
        assert_eq!(
            format_filter_summary(
                &SourceScope::all_enabled(true, true, true),
                PackageStateFilter::UpdatesAvailable,
                true,
                true,
                true
            ),
            Some("Updates Available".to_string())
        );
    }
}
