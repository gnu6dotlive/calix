pub mod dependencies;
pub mod files_build;
pub mod overview;

use crate::backend::models::{AlpmPackage, UnifiedPackage};
use crate::components::inspector_header::{
    InspectorHeader, InspectorHeaderProps, TabSelectHandler, WindowActionHandler,
};
use crate::components::semantic_value::StringActionHandler;
use crate::state::{AlsoIn, InspectorTab, PackageCapabilities};
use crate::theme::Theme;
use crate::views::inspector::dependencies::{DependenciesView, DependenciesViewProps};
use crate::views::inspector::files_build::{FilesBuildView, FilesBuildViewProps};
use crate::views::inspector::overview::{OverviewView, OverviewViewProps};
use gpui::*;

/// La plaque au repos : ce que la plaque de nom inscrit quand rien n'est
/// sélectionné.
///
/// Elle n'est pas un message d'attente. Une plaque qui s'excuse au centre
/// d'un vide se lit comme un panneau qui n'a pas chargé ; une inscription à
/// gauche, avec du vide au-dessous, se lit comme une plaque. Le sujet est le
/// canal que la banque met en avant, et le détail dit ce qu'il porte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuietPlate {
    pub heading: &'static str,
    pub subject: String,
    pub detail: String,
}

pub struct PackageInspectorProps<'a> {
    pub package: Option<&'a UnifiedPackage>,
    pub alpm_details: Option<&'a AlpmPackage>,
    /// The other sources where the same name is found.
    pub also_in: &'a [AlsoIn],
    pub detail_error: Option<&'a str>,
    pub pkgbuild: Option<&'a String>,
    pub is_loading_pkgbuild: bool,
    pub active_tab: InspectorTab,
    pub theme: &'a Theme,
    pub is_busy: bool,
    pub copy_feedback: bool,
    /// Ce que la plaque inscrit quand aucun paquet n'est ouvert.
    pub quiet: QuietPlate,
    pub on_select_tab: TabSelectHandler,
    pub on_install: Option<WindowActionHandler>,
    pub on_remove: Option<WindowActionHandler>,
    pub on_copy_install_cmd: Option<WindowActionHandler>,
    pub on_copy_pkgbuild: Option<WindowActionHandler>,
    pub on_navigate_package: Option<StringActionHandler>,
    pub on_retry_details: Option<WindowActionHandler>,
}

pub struct PackageInspectorView;

/// Le voyage de la coupe : 32 px, comme la coupe du périmètre.
const TAB_CUT_TRAVEL: f32 = 32.0;

impl PackageInspectorView {
    pub fn render_with_motion(
        props: PackageInspectorProps,
        reduce_motion: bool,
        tab_epoch: u64,
        tab_direction: f32,
    ) -> AnyElement {
        let theme = props.theme;

        let Some(pkg) = props.package else {
            let quiet = props.quiet;
            return div()
                .id("quiet_nameplate")
                .flex()
                .flex_col()
                .items_start()
                .justify_between()
                .size_full()
                .px(px(24.0))
                .py(px(20.0))
                .bg(theme.bg_app)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(5.0))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_muted)
                                .child(quiet.heading),
                        )
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(quiet.subject),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.text_secondary)
                                .child(quiet.detail),
                        ),
                )
                // La seule phrase qui enseigne le geste, au bas de la plaque,
                // là où une légende se tient.
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Select a row above and it opens here."),
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
                also_in: props.also_in,
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

        // La bascule de panneau est une coupe à la courbe : le panneau
        // sortant accélère, la coupe tombe au milieu du mouvement, et le
        // panneau entrant continue dans la même direction et décélère.
        //
        // Le geste est **la position**, pas l'opacité. Un fondu ne ferait
        // qu'effacer le voyage : le panneau se pose décalé du côté où l'onglet
        // est allé, et vient à sa place. L'opacité qui s'allumait au milieu
        // du chemin avalait les deux tiers du déplacement avant qu'il ne soit
        // visible — le voyage était là, mais personne ne le voyait.
        //
        // Le décalage est un `left` : l'inset déplace l'élément sans toucher
        // au flux, donc le corps se pose décalé sans réajuster son texte
        // (voir `state::motion::cut_travel`).
        let animated_body = if reduce_motion {
            div()
                .id("inspector_tab_body")
                .child(body)
                .into_any_element()
        } else {
            div()
                .id("inspector_tab_body")
                // The handle the proof measures: the harness reads the bounds
                // of the body under this name. A noop outside the tests.
                .debug_selector(|| "inspector_tab_body".to_string())
                .child(body)
                .with_animation(
                    ("tab-cut", tab_epoch as usize),
                    Animation::new(crate::state::MotionDurations::STANDARD)
                        .with_easing(crate::state::motion::ease_out),
                    move |elem, delta| {
                        let (opacity, offset) = crate::state::motion::cut_entry(delta, TAB_CUT_TRAVEL);
                        elem.opacity(opacity).left(px(tab_direction * offset))
                    },
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
    // The imports are explicit, not `use super::*`: a glob would bring gpui's
    // own `test` attribute into the module, and every `#[test]` would then
    // resolve to it — the macro would expand into itself and the compiler
    // would die of a stack overflow.
    use super::{PackageInspectorProps, PackageInspectorView, QuietPlate, TAB_CUT_TRAVEL};
    use crate::backend::models::{AlpmPackage, UnifiedPackage, UnifiedPackageSource};
    use crate::state::session::InspectorTab;
    use crate::theme::Theme;

    #[core::prelude::v1::test]
    fn test_inspector_empty_state_and_populated_render() {
        let theme = Theme::dark();
        let on_tab: crate::components::inspector_header::TabSelectHandler =
            std::rc::Rc::new(|_, _, _| {});

        // 1. Empty state
        let empty_props = PackageInspectorProps {
            package: None,
            alpm_details: None,
            also_in: &[],
            detail_error: None,
            pkgbuild: None,
            is_loading_pkgbuild: false,
            active_tab: InspectorTab::Overview,
            theme: &theme,
            is_busy: false,
            copy_feedback: false,
            quiet: QuietPlate {
                heading: "THE BANK",
                subject: "Official / ALPM".to_string(),
                detail: "official repositories — binaries · 1914 rows in this scope".to_string(),
            },
            on_select_tab: on_tab.clone(),
            on_install: None,
            on_remove: None,
            on_copy_install_cmd: None,
            on_copy_pkgbuild: None,
            on_navigate_package: None,
            on_retry_details: None,
        };
        let _empty_el = PackageInspectorView::render_with_motion(empty_props, true, 0, 1.0);

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
            also_in: &[],
            detail_error: Some("Simulated detail error"),
            pkgbuild: None,
            is_loading_pkgbuild: false,
            active_tab: InspectorTab::Overview,
            theme: &theme,
            is_busy: false,
            copy_feedback: false,
            // La plaque au repos n'est pas rendue quand un paquet est ouvert,
            // mais la charge est toujours fournie : une seule forme de props.
            quiet: QuietPlate {
                heading: "THE BANK",
                subject: "Official / ALPM".to_string(),
                detail: String::new(),
            },
            on_select_tab: on_tab,
            on_install: None,
            on_remove: None,
            on_copy_install_cmd: None,
            on_copy_pkgbuild: None,
            on_navigate_package: None,
            on_retry_details: Some(std::rc::Rc::new(|_, _| {})),
        };
        let _pop_el = PackageInspectorView::render_with_motion(populated_props, false, 1, 1.0);
    }

    /// La coupe de la plaque, mesurée sur l'image : le corps de l'onglet se
    /// pose décalé du côté où l'onglet est allé, et vient à sa place.
    ///
    /// La première image est prise dans l'image même où l'animation naît —
    /// l'état de l'animation est créé au premier placement, donc le premier
    /// écart vaut zéro — et la seconde après le voyage.
    ///
    /// C'est la preuve que le décalage atteint le dessin. La règle
    /// (`cut_entry`) dit de combien le corps se décale ; ce test dit que la
    /// fenêtre le dessine décalé, et de combien.
    #[gpui::test]
    fn the_tab_body_lands_displaced_and_comes_to_its_place(
        cx: &mut gpui::TestAppContext,
    ) {
        use gpui::prelude::*;
        use gpui::{point, px, size, Window};

        struct Fixture {
            theme: Theme,
            pkg: UnifiedPackage,
        }

        impl Render for Fixture {
            fn render(
                &mut self,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let theme = self.theme.clone();
                let on_tab: crate::components::inspector_header::TabSelectHandler =
                    std::rc::Rc::new(|_, _, _| {});
                let props = PackageInspectorProps {
                    package: Some(&self.pkg),
                    alpm_details: None,
                    also_in: &[],
                    detail_error: None,
                    pkgbuild: None,
                    is_loading_pkgbuild: false,
                    active_tab: InspectorTab::Overview,
                    theme: &theme,
                    is_busy: false,
                    copy_feedback: false,
                    quiet: QuietPlate {
                        heading: "THE BANK",
                        subject: "Official / ALPM".to_string(),
                        detail: String::new(),
                    },
                    on_select_tab: on_tab,
                    on_install: None,
                    on_remove: None,
                    on_copy_install_cmd: None,
                    on_copy_pkgbuild: None,
                    on_navigate_package: None,
                    on_retry_details: None,
                };
                PackageInspectorView::render_with_motion(props, false, 1, 1.0)
            }
        }

        let cx = cx.add_empty_window();
        let fixture = cx.new(|_| Fixture {
            theme: Theme::dark(),
            pkg: UnifiedPackage {
                name: "ripgrep".into(),
                version: "14.1.0-1".into(),
                description: "fast search".into(),
                source_type: "ALPM".into(),
                repository_or_remote: "extra".into(),
                is_installed: true,
                has_update: false,
                new_version: None,
                inner: UnifiedPackageSource::Standard(AlpmPackage::default()),
            },
        });

        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.draw(
                point(px(0.0), px(0.0)),
                size(px(900.0), px(600.0)),
                |_window, _cx| fixture.clone().into_element(),
            );
        };

        draw(cx);
        let departure = cx
            .debug_bounds("inspector_tab_body")
            .expect("le corps de l'onglet est dessiné");

        // La coupe dure 160 ms ; on attend plus, et on dessine encore.
        std::thread::sleep(std::time::Duration::from_millis(260));
        draw(cx);
        let arrival = cx
            .debug_bounds("inspector_tab_body")
            .expect("le corps de l'onglet est dessiné");

        // Le corps est arrivé du côté où l'onglet est allé (la direction est
        // +1) : il s'est posé 32 px plus à droite, puis il a pris sa place.
        assert_eq!(
            departure.origin.x - arrival.origin.x,
            px(TAB_CUT_TRAVEL),
            "le corps s'est posé décalé de 32 px, et il est revenu à sa place"
        );
        assert!(
            departure.origin.x > arrival.origin.x,
            "le décalage va dans le sens de la coupe"
        );
    }
}
