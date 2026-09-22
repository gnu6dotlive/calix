use crate::render_lab::catalog::FixtureCatalog;
use crate::render_lab::confront::{
    comparison_canvas_spec, recipe_for, render_confrontation, CONFRONT_BOX_HEIGHT_PX,
    FIDUCIAL_MARKER_RGB,
};
use crate::render_lab::experimental::shader_present_probe;
use crate::render_lab::fixture::{FixtureGroup, FixtureId};
use crate::render_lab::ledger::CapabilityLedger;
use crate::render_lab::state::{ClockMode, RenderLabState};
use crate::theme::Theme;
use crate::visual_style::paint::apply_surface_projection;
use crate::visual_style::profile::ColorScheme;
use crate::visual_style::resolver::resolve_style;
use crate::visual_style::roles::SurfaceRole;
use crate::visual_style::VisualStyleId;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub type SelectFixtureHandler = Rc<dyn Fn(FixtureId, &mut Window, &mut App) + 'static>;
pub type ClearFixtureHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct RenderLabViewProps<'a> {
    pub state: &'a RenderLabState,
    pub theme: &'a Theme,
    pub on_select_fixture: SelectFixtureHandler,
    pub on_clear_fixture: ClearFixtureHandler,
    pub on_back_settings: WindowActionHandler,
    pub on_toggle_ledger: WindowActionHandler,
    pub ledger_collapsed: bool,
    pub catalog_scroll_handle: &'a ScrollHandle,
}

struct MatrixRoleRow {
    role: SurfaceRole,
    treatment_label: &'static str,
    role_sublabel: &'static str,
    bg_fn: fn(&Theme) -> Rgba,
}

pub struct RenderLabView;

impl RenderLabView {
    pub fn render(props: RenderLabViewProps) -> impl IntoElement {
        let theme = *props.theme;
        let active_id = props.state.active_fixture.as_ref();
        let selected_def = active_id.and_then(FixtureCatalog::find);

        let mut measurement_stage = div()
            .id("render_lab_measurement_stage")
            .w_full()
            .flex_none()
            .flex()
            .justify_center()
            .py_2();
        if let Some(f) = selected_def {
            if let Some(spec) = comparison_canvas_spec(f) {
                let mut controlled = div()
                    .w(px(spec.width_px as f32 + 4.0))
                    .h(px(spec.height_px as f32 + 4.0))
                    .flex_none()
                    .border_1()
                    .border_color(rgb(FIDUCIAL_MARKER_RGB));
                if let Some(content) = render_confrontation(f) {
                    controlled = controlled.child(content);
                }
                measurement_stage = measurement_stage.child(controlled);
            }
        }
        measurement_stage = measurement_stage.child(Self::render_shader_experiment(&theme));

        let body = div()
            .id("render_lab_body")
            .flex_1()
            .flex()
            .flex_row()
            .overflow_hidden()
            // ── Column 1: Catalog Column (260px) ───────────────────────────
            .child(Self::render_catalog(
                &props,
                &theme,
                active_id,
                props.catalog_scroll_handle,
            ))
            // ── Column 2: Preview / Fixture Specification (flex-1) ─────────
            .child(Self::render_preview(&props, &theme, selected_def))
            // ── Column 3: State Ledger & Diagnostics (280px) ───────────────
            .child(Self::render_inspector(
                props.state,
                &theme,
                props.ledger_collapsed,
                props.on_toggle_ledger,
            ));

        let on_back_key = props.on_back_settings.clone();
        let on_back = props.on_back_settings;
        let header_focus = theme.border_focus;
        let header = div()
            .id("render_lab_header")
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.bg_surface)
            .child(
                div()
                    .id("render_lab_back_settings")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(header_focus))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_back_key(window, cx);
                        }
                    })
                    .px_2()
                    .py_1()
                    .rounded_md()
                    .text_xs()
                    .text_color(theme.text_secondary)
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.bg_surface_hover))
                    .child("← Back to Settings")
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        on_back(window, cx);
                    }),
            )
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .child("Render Lab"),
            );

        div()
            .id("render_lab_root")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.bg_app)
            .text_color(theme.text_primary)
            .child(header)
            .child(measurement_stage)
            .child(body)
    }

    fn render_shader_experiment(theme: &Theme) -> impl IntoElement {
        let mut probe = div()
            .id("render04_i03_shader_probe")
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.warning_text)
                    .child("EXPERIMENTAL · experiment.render04-i03-shader-present"),
            );
        match shader_present_probe() {
            Ok(output) => {
                probe = probe.child(
                    img(output.image.clone())
                        .size(px(256.0))
                        .object_fit(ObjectFit::Contain),
                );
            }
            Err(error) => {
                probe = probe.child(
                    div()
                        .text_xs()
                        .text_color(theme.warning_text)
                        .child(format!("CPU readback blocked: {error}")),
                );
            }
        }
        probe
    }

    fn render_catalog(
        props: &RenderLabViewProps,
        theme: &Theme,
        active_id: Option<&FixtureId>,
        scroll_handle: &ScrollHandle,
    ) -> impl IntoElement {
        let mut groups_col = div()
            .id("render_lab_catalog")
            .w(px(260.0))
            .h_full()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.bg_surface)
            .overflow_y_scroll()
            .track_scroll(scroll_handle);

        // Header
        groups_col = groups_col.child(
            div()
                .p_4()
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .child("Fixture Catalog"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child(format!("{} canonical fixtures", FixtureCatalog::count())),
                ),
        );

        // Style Projection Matrix toggle button
        let is_matrix_active = active_id.is_none();
        let on_clear = props.on_clear_fixture.clone();
        groups_col = groups_col.child(
            div()
                .id("cat_btn_style_matrix")
                .px_4()
                .py_2()
                .mx_2()
                .mt_2()
                .rounded_md()
                .cursor_pointer()
                .bg(if is_matrix_active {
                    theme.bg_surface_active
                } else {
                    theme.bg_surface
                })
                .hover(move |s| s.bg(theme.bg_surface_hover))
                .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                    on_clear(window, cx);
                })
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_xs()
                        .font_weight(if is_matrix_active {
                            FontWeight::BOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(if is_matrix_active {
                            theme.accent
                        } else {
                            theme.text_primary
                        })
                        .child("⊞ Style Matrix (STYLE-00B)"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("16-cell projection & contrast"),
                ),
        );

        for group in FixtureGroup::ALL {
            let fixtures: Vec<_> = FixtureCatalog::for_group(*group).collect();

            let mut group_section = div().flex().flex_col().py_2();

            // Group header
            group_section = group_section.child(
                div()
                    .px_4()
                    .py_1()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.text_secondary)
                            .child(group.label()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_muted)
                            .child(format!("Board {}", group.board_code())),
                    ),
            );

            // Fixture rows
            for f in fixtures {
                let is_selected = active_id == Some(&f.id);
                let on_select = props.on_select_fixture.clone();
                let f_id = f.id.clone();

                let row_bg = if is_selected {
                    theme.bg_surface_active
                } else {
                    theme.bg_surface
                };

                let row_text_color = if is_selected {
                    theme.accent
                } else {
                    theme.text_primary
                };

                let display_title = f.provenance.source_label.unwrap_or_else(|| f.id.as_str());

                let row = div()
                    .id(SharedString::from(format!("cat_row_{}", f.id.as_str())))
                    .px_4()
                    .py_1p5()
                    .mx_2()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(row_bg)
                    .hover(move |s| s.bg(theme.bg_surface_hover))
                    .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        on_select(f_id.clone(), window, cx);
                    })
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(if is_selected {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(row_text_color)
                            .child(display_title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(format!("#{} · seed {}", f.provenance.cell_index, f.seed)),
                    );

                group_section = group_section.child(row);
            }

            groups_col = groups_col.child(group_section);
        }

        groups_col
    }

    fn render_preview(
        props: &RenderLabViewProps,
        theme: &Theme,
        selected_def: Option<&'static crate::render_lab::fixture::FixtureDef>,
    ) -> impl IntoElement {
        let mut preview = div()
            .id("render_lab_preview")
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_scroll()
            .p_6();

        match selected_def {
            Some(f) => {
                let on_clear = props.on_clear_fixture.clone();
                // Header
                preview = preview.child(
                    div()
                        .mb_6()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(FontWeight::BOLD)
                                        .child(f.id.as_str()),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(theme.bg_surface_hover)
                                        .border_1()
                                        .border_color(theme.border)
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child(format!("Board {}", f.provenance.board)),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(theme.bg_surface)
                                        .border_1()
                                        .border_color(theme.border)
                                        .cursor_pointer()
                                        .hover(move |s| s.bg(theme.bg_surface_hover))
                                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                                            on_clear(window, cx);
                                        })
                                        .text_xs()
                                        .text_color(theme.accent)
                                        .child("⊞ Show Style Matrix"),
                                ),
                        )
                        .child(
                            div()
                                .mt_1()
                                .text_sm()
                                .text_color(theme.text_secondary)
                                .child(if !f.description.is_empty() {
                                    f.description
                                } else {
                                    "Unlabeled source microstructure swatch"
                                }),
                        ),
                );

                // Confrontation Box: fixed-height stock render framed by the
                // fiducial marker the determinism harness locates in captures.
                let observation = CapabilityLedger::find(f.id.as_str());
                let canvas = comparison_canvas_spec(f);
                let mut confront_box = div()
                    .when_some(canvas, |element, spec| {
                        element
                            .w(px(spec.width_px as f32 + 4.0))
                            .h(px(spec.height_px as f32 + 4.0))
                    })
                    .when(canvas.is_none(), |element| {
                        element.w_full().h(px(CONFRONT_BOX_HEIGHT_PX))
                    })
                    .when(canvas.is_some(), |element| element.flex_none())
                    .border_1()
                    .border_color(if canvas.is_some() {
                        theme.border
                    } else {
                        rgb(FIDUCIAL_MARKER_RGB)
                    });
                match render_confrontation(f) {
                    Some(content) => {
                        confront_box = confront_box.child(content);
                    }
                    None => {
                        confront_box = confront_box
                            .bg(theme.bg_surface)
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.text_secondary)
                                    .child("No confrontation recipe yet"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.text_muted)
                                    .child("Unevaluated in RENDER-01"),
                            );
                    }
                }
                preview = preview.child(confront_box);

                // Observed verdict badge: ledger observation when recorded,
                // unevaluated otherwise. Never the catalog default.
                let (badge_text, badge_color) = match observation {
                    Some(obs) => (
                        format!("Observed: {} · {}", obs.verdict, obs.recipe),
                        theme.accent,
                    ),
                    None => (
                        "Capability: UNKNOWN (unevaluated)".to_string(),
                        theme.warning_text,
                    ),
                };
                preview = preview.child(
                    div().mt_3().flex().flex_row().child(
                        div()
                            .px_3()
                            .py_1()
                            .rounded_full()
                            .border_1()
                            .border_color(theme.border)
                            .bg(theme.bg_app)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(badge_color)
                            .child(badge_text),
                    ),
                );

                // Provenance Card
                preview = preview.child(
                    div()
                        .mt_6()
                        .p_4()
                        .rounded_lg()
                        .border_1()
                        .border_color(theme.border)
                        .bg(theme.bg_surface)
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_secondary)
                                .child("PROVENANCE & INTEGRITY"),
                        )
                        .child(Self::meta_row(
                            theme,
                            "Source Board",
                            &format!(
                                "Board {} (Cell {})",
                                f.provenance.board, f.provenance.cell_index
                            ),
                        ))
                        .child(Self::meta_row(
                            theme,
                            "Source Label",
                            f.provenance.source_label.unwrap_or("None (unlabeled)"),
                        ))
                        .child(Self::meta_row(
                            theme,
                            "Artifact",
                            f.provenance.source_artifact,
                        ))
                        .child(Self::meta_row(theme, "SHA-256", f.provenance.source_sha256))
                        .child(Self::meta_row(theme, "Seed", &format!("{}", f.seed)))
                        .child(Self::meta_row(theme, "Determinism", "Deterministic"))
                        .child(Self::meta_row(
                            theme,
                            "Catalog Capability",
                            f.capability.as_str(),
                        ))
                        .child(Self::meta_row(
                            theme,
                            "Recipe",
                            recipe_for(f).map(|r| r.id).unwrap_or("none yet"),
                        ))
                        .child(Self::meta_row(
                            theme,
                            "Observed Capability",
                            observation
                                .map(|o| o.verdict.as_str())
                                .unwrap_or("unevaluated"),
                        )),
                );
            }
            None => {
                preview = preview.child(Self::render_style_matrix(theme));
            }
        }

        preview
    }

    fn render_style_matrix(theme: &Theme) -> impl IntoElement {
        let roles = [
            MatrixRoleRow {
                role: SurfaceRole::NavigationRail,
                treatment_label: "Ambient Chrome",
                role_sublabel: "NavigationRail (sidebar.rs)",
                bg_fn: |t: &Theme| t.bg_sidebar,
            },
            MatrixRoleRow {
                role: SurfaceRole::QueryChrome,
                treatment_label: "Interactive Chrome",
                role_sublabel: "QueryChrome (query_workbench.rs)",
                bg_fn: |t: &Theme| t.bg_sidebar,
            },
            MatrixRoleRow {
                role: SurfaceRole::ResultSurface,
                treatment_label: "Content Plane",
                role_sublabel: "ResultSurface (package_card.rs)",
                bg_fn: |t: &Theme| t.bg_surface,
            },
            MatrixRoleRow {
                role: SurfaceRole::Menu,
                treatment_label: "Elevated Surface",
                role_sublabel: "Menu (menu.rs)",
                bg_fn: |t: &Theme| t.bg_surface,
            },
        ];

        let columns = [
            (VisualStyleId::Standard, Theme::dark(), "Standard Dark"),
            (VisualStyleId::Standard, Theme::light(), "Standard Light"),
            (
                VisualStyleId::Transparency,
                Theme::dark(),
                "Transparency Dark",
            ),
            (
                VisualStyleId::Transparency,
                Theme::light(),
                "Transparency Light",
            ),
        ];

        let mut matrix_col = div().flex().flex_col().gap_4().w_full();

        // ── 1. Title Banner & Legend ─────────────────────────────────────────
        matrix_col = matrix_col.child(
            div()
                .p_4()
                .rounded_lg()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(theme.border)
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .child("STYLE-00B: Surface Projection & Transparency Matrix"),
                        )
                        .child(
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.accent)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x000000))
                                .child("16 / 16 Proven Cells"),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .child("Stock GPUI 0.2.2 deterministic surface projection: 4 Surface Treatments × 2 Color Schemes × 2 Visual Styles."),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .mt_1()
                        .child(Self::telemetry_pill(theme, "Depth", "NATIVE (h01)"))
                        .child(Self::telemetry_pill(theme, "Rim", "NATIVE (h07)"))
                        .child(Self::telemetry_pill(theme, "Microstructure", "TEXTURE_PROOF (K01-K14)"))
                        .child(Self::telemetry_pill(theme, "Protection", "SCRIM FLOOR ≥ 4.5:1")),
                ),
        );

        // ── 2. The 16-Cell Grid ──────────────────────────────────────────────
        // Header row
        let mut header_row = div().flex().flex_row().gap_3().items_center().pb_1();

        // Corner cell
        header_row = header_row.child(
            div()
                .w(px(160.0))
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(theme.text_muted)
                .child("TREATMENT / ROLE"),
        );

        for (_, _, col_label) in &columns {
            header_row = header_row.child(
                div()
                    .flex_1()
                    .min_w(px(180.0))
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_center()
                    .text_color(theme.text_primary)
                    .child(*col_label),
            );
        }
        matrix_col = matrix_col.child(header_row);

        // Grid rows
        for row in &roles {
            let mut row_div = div().flex().flex_row().gap_3().items_start();

            // Row header
            row_div = row_div.child(
                div()
                    .w(px(160.0))
                    .h(px(120.0))
                    .p_3()
                    .rounded_md()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .flex()
                    .flex_col()
                    .justify_center()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(row.treatment_label),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_muted)
                            .child(row.role_sublabel),
                    ),
            );

            // 4 Cells for this row
            for (style, cell_theme, _) in &columns {
                let base_bg = (row.bg_fn)(cell_theme);
                let scheme = ColorScheme::from_dark_theme(cell_theme.is_dark());
                let projection = resolve_style(*style, row.role, scheme);
                let paint = &projection.paint;

                // High-contrast textured backdrop to prove transparency and content scrim
                let backdrop = div()
                    .absolute()
                    .size_full()
                    .flex()
                    .flex_row()
                    .child(div().flex_1().h_full().bg(rgb(0x0284c7))) // Sky Blue
                    .child(div().flex_1().h_full().bg(rgb(0xe11d48))); // Rose

                let surface_base = div()
                    .relative()
                    .size_full()
                    .p_3()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .rounded_md()
                    .border_1()
                    .border_color(cell_theme.border);

                let styled_surface =
                    apply_surface_projection(surface_base, row.role, *style, base_bg, cell_theme);

                let opacity_pct = (paint.surface_opacity * 100.0) as u32;
                let scrim_pct = (paint.scrim_floor_opacity * 100.0) as u32;

                let cell_content = styled_surface
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cell_theme.text_primary)
                                    .child(if *style == VisualStyleId::Standard {
                                        "Standard"
                                    } else {
                                        "Transparency"
                                    }),
                            )
                            .child(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(if cell_theme.is_dark() {
                                        rgb(0x1e293b)
                                    } else {
                                        rgb(0xe2e8f0)
                                    })
                                    .text_xs()
                                    .text_color(cell_theme.text_primary)
                                    .child(if cell_theme.is_dark() {
                                        "Dark"
                                    } else {
                                        "Light"
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .py_1()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(cell_theme.text_primary)
                                    .child("Shelly Content Plane"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cell_theme.text_secondary)
                                    .child("Legibility WCAG AA ≥ 4.5:1"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .text_xs()
                            .text_color(cell_theme.text_muted)
                            .child(format!("{opacity_pct}% opac · {scrim_pct}% scrim"))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .child(if paint.rim_active { "rim " } else { "" })
                                    .child(if paint.shadow_active { "shadow" } else { "" }),
                            ),
                    );

                let cell_container = div()
                    .relative()
                    .flex_1()
                    .min_w(px(180.0))
                    .h(px(120.0))
                    .rounded_md()
                    .overflow_hidden()
                    .border_1()
                    .border_color(theme.border)
                    .child(backdrop)
                    .child(cell_content);

                row_div = row_div.child(cell_container);
            }

            matrix_col = matrix_col.child(row_div);
        }

        matrix_col
    }

    fn telemetry_pill(theme: &Theme, label: &'static str, val: &'static str) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .rounded_md()
            .bg(theme.bg_app)
            .border_1()
            .border_color(theme.border)
            .flex()
            .flex_row()
            .gap_1p5()
            .items_center()
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child(label),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.accent)
                    .child(val),
            )
    }

    fn render_inspector(
        state: &RenderLabState,
        theme: &Theme,
        collapsed: bool,
        on_toggle: WindowActionHandler,
    ) -> impl IntoElement {
        let (clock_str, clock_detail) = match state.clock {
            ClockMode::Realtime => ("Realtime", "Running at wall clock".to_string()),
            ClockMode::Frozen(t) => ("Frozen", format!("t = {t:.3}s")),
        };

        let on_toggle_key = on_toggle.clone();
        let on_toggle_mouse = on_toggle;
        let focus_border = theme.border_focus;
        let ledger_content = div()
            .flex()
            .flex_col()
            .gap_4()
            .w_full()
            .child(
                div()
                    .p_3()
                    .rounded_md()
                    .bg(theme.bg_app)
                    .border_1()
                    .border_color(theme.border)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_secondary)
                            .child("STATE AXES"),
                    )
                    .child(Self::meta_row(
                        theme,
                        "Topology",
                        state.active_topology.as_str(),
                    ))
                    .child(Self::meta_row(
                        theme,
                        "Motion",
                        state.active_motion.as_str(),
                    ))
                    .child(Self::meta_row(
                        theme,
                        "Quality",
                        state.active_quality.as_str(),
                    ))
                    .child(Self::meta_row(theme, "Style", state.active_style.as_str()))
                    .child(Self::meta_row(theme, "Clock Mode", clock_str))
                    .child(Self::meta_row(theme, "Clock Time", &clock_detail)),
            )
            .child(
                div()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_secondary)
                            .mb_2()
                            .child("DIAGNOSTICS"),
                    )
                    .child({
                        let mut diag_col = div().flex().flex_col().gap_1();
                        for d in &state.diagnostics {
                            diag_col = diag_col.child(
                                div()
                                    .p_2()
                                    .rounded_sm()
                                    .bg(theme.bg_app)
                                    .border_1()
                                    .border_color(theme.border)
                                    .text_xs()
                                    .child(d.message.clone()),
                            );
                        }
                        diag_col
                    }),
            );

        div()
            .id("render_lab_inspector")
            .w(px(if collapsed { 40.0 } else { 280.0 }))
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .border_l_1()
            .border_color(theme.border)
            .bg(theme.bg_surface)
            .overflow_scroll()
            .when(!collapsed, |element| element.p_4())
            .child(
                div()
                    .id("render_lab_ledger_toggle")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_toggle_key(window, cx);
                        }
                    })
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        on_toggle_mouse(window, cx);
                    })
                    .flex()
                    .w_full()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.bg_surface_hover))
                    .child(
                        svg()
                            .path(if collapsed {
                                crate::icons::AppIcon::Expand.path()
                            } else {
                                crate::icons::AppIcon::Collapse.path()
                            })
                            .size(px(16.0))
                            .text_color(theme.text_secondary),
                    )
                    .when(!collapsed, |element| {
                        element.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .child("Render Lab Ledger"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.text_muted)
                                        .child("Read-only control state (CLI-driven)"),
                                ),
                        )
                    }),
            )
            .when(!collapsed, |element| element.child(ledger_content))
    }

    fn meta_row(theme: &Theme, label: &'static str, val: &str) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .justify_between()
            .items_start()
            .gap_2()
            .text_xs()
            .child(
                div()
                    .text_color(theme.text_muted)
                    .flex_shrink_0()
                    .child(label),
            )
            .child(
                div()
                    .text_color(theme.text_primary)
                    .font_weight(FontWeight::MEDIUM)
                    .text_right()
                    .child(val.to_string()),
            )
    }
}
