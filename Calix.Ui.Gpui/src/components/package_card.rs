use crate::backend::models::UnifiedPackage;
use crate::components::package_identity::PackageIdentity;
use crate::components::package_table::PackageTable;
use crate::theme::Theme;
use crate::ui_metrics::UiMetrics;
use crate::visual_style::paint::apply_surface_projection;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct PackageCardProps<'a> {
    pub package: &'a UnifiedPackage,
    pub is_selected: bool,
    pub theme: &'a Theme,
    pub compact: bool,
    pub visual_style: VisualStyleId,
}

pub struct PackageCard;

impl PackageCard {
    pub fn render(props: PackageCardProps) -> impl IntoElement {
        let theme = props.theme;
        let pkg = props.package;
        let is_selected = props.is_selected;

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

        let card_height = if props.compact {
            UiMetrics::CARD_HEIGHT_COMPACT
        } else {
            UiMetrics::CARD_HEIGHT_NORMAL
        };

        let avatar_size = if props.compact { 28.0 } else { 36.0 };
        let avatar_radius = if props.compact { 4.0 } else { 6.0 };
        let display_size = PackageTable::display_size(pkg);
        let has_size = display_size != "—";

        // Dérivation de la source et du dépôt sans badge en forme de pilule
        let source_label = match pkg.source_type.to_uppercase().as_str() {
            "ALPM" => "Arch",
            "AUR" => "AUR",
            "FLATPAK" => "Flatpak",
            "APPIMAGE" => "AppImage",
            _ => pkg.source_type.as_str(),
        };

        let show_repo = !pkg.repository_or_remote.is_empty()
            && !pkg
                .repository_or_remote
                .eq_ignore_ascii_case(&pkg.source_type)
            && !pkg.repository_or_remote.eq_ignore_ascii_case(source_label);

        let card = div()
            .h(px(card_height))
            .w_full()
            .flex()
            .items_start()
            .gap_3()
            .px_3()
            .py(if props.compact { px(4.0) } else { px(6.0) })
            .rounded_md()
            .border_1()
            .border_color(border_color)
            .cursor_pointer()
            .overflow_hidden();

        apply_surface_projection(
            card,
            SurfaceRole::ResultSurface,
            props.visual_style,
            bg_color,
            theme,
        )
        // Barre d'accentuation latérale pour le paquet sélectionné
        .when(is_selected, |el| el.border_l_4().border_color(theme.accent))
        // Rétroaction tactile au survol
        .when(!is_selected, |el| {
            let hover_bg = theme.bg_surface_hover;
            let hover_border = theme.border_focus;
            el.hover(move |s| s.bg(hover_bg).border_color(hover_border))
        })
        // Ancre visuelle : Avatar d'identité de paquet résolu selon la chaîne à 3 tiers
        .child(PackageIdentity::render_avatar(
            pkg,
            avatar_size,
            avatar_radius,
            theme,
        ))
        // Colonne de contenu structurée
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .flex()
                .flex_col()
                .justify_between()
                .overflow_hidden()
                // Ligne 1 : En-tête (Nom à gauche, Version + Taille à droite en monospace)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_weight(FontWeight::BOLD)
                                .text_sm()
                                .text_color(if is_selected {
                                    theme.accent
                                } else {
                                    theme.text_primary
                                })
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(pkg.name.clone()),
                        )
                        .child(
                            div()
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .gap_2()
                                .when(pkg.has_update, |el| {
                                    let new_ver = pkg.new_version.clone().unwrap_or_default();
                                    el.child(
                                        div()
                                            .text_xs()
                                            .font_family("monospace")
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.warning_text)
                                            .child(format!("→ {}", new_ver)),
                                    )
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family("monospace")
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.text_muted)
                                        .child(pkg.version.clone()),
                                )
                                .when(has_size, |el| {
                                    el.child(
                                        div()
                                            .text_xs()
                                            .font_family("monospace")
                                            .font_weight(FontWeight::NORMAL)
                                            .text_color(theme.text_secondary)
                                            .child(display_size),
                                    )
                                }),
                        ),
                )
                // Ligne 2 : Métadonnées desktop calmes sans pilules colorées ("Arch · extra · Installed")
                .child({
                    let mut meta = div().flex().items_center().gap_1p5().text_xs().child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.text_secondary)
                            .child(source_label.to_string()),
                    );

                    if show_repo {
                        meta = meta
                            .child(div().text_color(theme.text_muted).child("·"))
                            .child(
                                div()
                                    .text_color(theme.text_muted)
                                    .child(pkg.repository_or_remote.clone()),
                            );
                    }

                    meta = meta.child(div().text_color(theme.text_muted).child("·"));

                    if pkg.has_update {
                        meta = meta.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .text_color(theme.warning_text)
                                .font_weight(FontWeight::MEDIUM)
                                .child(div().size(px(5.0)).rounded_full().bg(theme.warning))
                                .child("Update available"),
                        );
                    } else if pkg.is_installed {
                        meta = meta.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .text_color(theme.success_text)
                                .font_weight(FontWeight::MEDIUM)
                                .child(div().size(px(5.0)).rounded_full().bg(theme.success))
                                .child("Installed"),
                        );
                    } else {
                        meta = meta.child(div().text_color(theme.text_muted).child("Available"));
                    }

                    meta
                })
                // Ligne 3 : Description aérée avec bornage strict multi-lignes
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .overflow_hidden()
                        .text_ellipsis()
                        .line_clamp(if props.compact { 1 } else { 2 })
                        .child(if pkg.description.is_empty() {
                            "No description available for this package.".to_string()
                        } else {
                            pkg.description.clone()
                        }),
                ),
        )
    }
}
