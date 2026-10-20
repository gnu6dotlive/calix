use crate::brand::Brand;
use crate::components::menu::{
    MenuCheckmarkItem, MenuKeyHandler, MenuLifecycle, MenuSurface, MenuSurfaceProps,
};
use crate::config::{ConfigManager, GpuiUiConfig, CalixSettings};
use crate::icons::AppIcon;
use crate::theme::Theme;
use crate::visual_style::VisualStyleId;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::rc::Rc;

pub type WindowActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
pub type StyleSelectHandler = Rc<dyn Fn(VisualStyleId, &mut Window, &mut App) + 'static>;
pub type BrandSelectHandler = Rc<dyn Fn(Brand, &mut Window, &mut App) + 'static>;

pub struct StyleSelectRowProps<'a> {
    pub current: VisualStyleId,
    pub menu_open: bool,
    pub menu_epoch: usize,
    pub menu_highlighted: usize,
    pub reduce_motion: bool,
    pub on_open: WindowActionHandler,
    pub on_close: WindowActionHandler,
    pub on_navigate: MenuKeyHandler,
    pub on_select: StyleSelectHandler,
    pub theme: &'a Theme,
}

pub struct SettingsViewProps<'a> {
    pub calix_settings: &'a CalixSettings,
    pub gpui_config: &'a GpuiUiConfig,
    pub is_dirty: bool,
    pub style_menu_open: bool,
    pub style_menu_epoch: usize,
    pub style_menu_highlighted: usize,
    pub advanced_settings_expanded: bool,
    pub theme: &'a Theme,
    pub on_toggle_aur: WindowActionHandler,
    pub on_toggle_flatpak: WindowActionHandler,
    pub on_toggle_appimage: WindowActionHandler,
    pub on_toggle_cascade_delete: WindowActionHandler,
    pub on_toggle_remove_configs: WindowActionHandler,
    pub on_toggle_dark_theme: WindowActionHandler,
    pub on_toggle_compact_view: WindowActionHandler,
    pub on_toggle_log_drawer_auto_open: WindowActionHandler,
    pub on_toggle_reduce_motion: WindowActionHandler,
    pub on_open_style_menu: WindowActionHandler,
    pub on_close_style_menu: WindowActionHandler,
    pub on_navigate_style_menu: MenuKeyHandler,
    pub on_select_style: StyleSelectHandler,
    pub on_select_brand: BrandSelectHandler,
    pub on_toggle_advanced_settings: WindowActionHandler,
    pub on_open_render_lab: WindowActionHandler,
    pub on_save: WindowActionHandler,
    pub on_reset: Option<WindowActionHandler>,
}

#[derive(Debug, Clone)]
pub struct SettingsView {
    pub draft_calix: CalixSettings,
    pub draft_gpui: GpuiUiConfig,
    pub is_dirty: bool,
    pub style_menu_open: bool,
    pub style_menu_epoch: usize,
    pub style_menu_highlighted: usize,
    pub advanced_settings_expanded: bool,
}

impl SettingsView {
    pub fn new(calix_settings: CalixSettings, gpui_config: GpuiUiConfig) -> Self {
        Self {
            draft_calix: calix_settings,
            draft_gpui: gpui_config,
            is_dirty: false,
            style_menu_open: false,
            style_menu_epoch: 0,
            style_menu_highlighted: 0,
            advanced_settings_expanded: false,
        }
    }

    fn style_menu_index(style: VisualStyleId) -> usize {
        match style {
            VisualStyleId::Standard => 0,
            VisualStyleId::Transparency => 1,
        }
    }

    fn style_at_index(index: usize) -> VisualStyleId {
        if index == 0 {
            VisualStyleId::Standard
        } else {
            VisualStyleId::Transparency
        }
    }

    pub fn open_style_menu(&mut self) {
        self.style_menu_open = true;
        self.style_menu_epoch += 1;
        self.style_menu_highlighted = Self::style_menu_index(self.draft_gpui.visual_style);
    }

    pub fn close_style_menu(&mut self) {
        self.style_menu_open = false;
    }

    pub fn navigate_style_menu(&mut self, key: &str) {
        match key {
            "up" => {
                self.style_menu_highlighted =
                    (self.style_menu_highlighted + 1) % VisualStyleId::ALL.len();
            }
            "down" => {
                self.style_menu_highlighted =
                    (self.style_menu_highlighted + 1) % VisualStyleId::ALL.len();
            }
            "home" => self.style_menu_highlighted = 0,
            "end" => self.style_menu_highlighted = VisualStyleId::ALL.len() - 1,
            "enter" | "space" => {
                let selected = Self::style_at_index(self.style_menu_highlighted);
                self.set_visual_style(selected);
            }
            _ => {}
        }
    }

    pub fn set_visual_style(&mut self, style: VisualStyleId) {
        self.draft_gpui.visual_style = style;
        self.is_dirty = true;
        self.style_menu_open = false;
    }

    pub fn toggle_advanced_settings(&mut self) {
        self.advanced_settings_expanded = !self.advanced_settings_expanded;
    }

    pub fn toggle_aur(&mut self) {
        self.draft_calix.aur_enabled = !self.draft_calix.aur_enabled;
        self.is_dirty = true;
    }

    pub fn toggle_flatpak(&mut self) {
        self.draft_calix.flat_pack_enabled = !self.draft_calix.flat_pack_enabled;
        self.is_dirty = true;
    }

    pub fn toggle_appimage(&mut self) {
        self.draft_calix.app_image_enabled = !self.draft_calix.app_image_enabled;
        self.is_dirty = true;
    }

    pub fn toggle_cascade_delete(&mut self) {
        self.draft_calix.package_management_cascade_delete =
            !self.draft_calix.package_management_cascade_delete;
        self.is_dirty = true;
    }

    pub fn toggle_remove_configs(&mut self) {
        self.draft_calix.package_management_remove_configs =
            !self.draft_calix.package_management_remove_configs;
        self.is_dirty = true;
    }

    pub fn toggle_dark_theme(&mut self) {
        self.draft_gpui.dark_theme = !self.draft_gpui.dark_theme;
        self.is_dirty = true;
    }

    /// Applique la couleur de marque au brouillon ; l'appelant repeint l'interface.
    pub fn set_brand_color(&mut self, brand: Brand) {
        self.draft_gpui.brand_color = brand;
        self.is_dirty = true;
    }

    pub fn toggle_compact_view(&mut self) {
        self.draft_gpui.compact_view = !self.draft_gpui.compact_view;
        self.is_dirty = true;
    }

    pub fn toggle_log_drawer_auto_open(&mut self) {
        self.draft_gpui.log_drawer_open = !self.draft_gpui.log_drawer_open;
        self.is_dirty = true;
    }

    pub fn toggle_reduce_motion(&mut self) {
        self.draft_gpui.reduce_motion = !self.draft_gpui.reduce_motion;
        self.is_dirty = true;
    }

    pub fn reset_to(&mut self, calix: CalixSettings, gpui: GpuiUiConfig) {
        self.draft_calix = calix;
        self.draft_gpui = gpui;
        self.is_dirty = false;
    }

    /// Sauvegarde pure paramétrable par injection de closures (permettant les tests unitaires sans disque)
    pub fn save_with<F1, F2>(&mut self, save_calix: F1, save_gpui: F2) -> anyhow::Result<()>
    where
        F1: FnOnce(&CalixSettings) -> anyhow::Result<()>,
        F2: FnOnce(&GpuiUiConfig) -> anyhow::Result<()>,
    {
        save_calix(&self.draft_calix)?;
        save_gpui(&self.draft_gpui)?;
        self.is_dirty = false;
        Ok(())
    }

    /// Sauvegarde effective vers le disque via ConfigManager
    pub fn save(&mut self) -> anyhow::Result<()> {
        self.save_with(
            ConfigManager::save_calix_settings,
            ConfigManager::save_gpui_config,
        )
    }

    pub fn render(props: SettingsViewProps) -> impl IntoElement {
        let theme = props.theme;
        let s = props.calix_settings;
        let g = props.gpui_config;
        let is_dirty = props.is_dirty;

        let mut root = div()
            .id("settings_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .p_6()
            .overflow_scroll();

        // En-tête des paramètres
        root = root.child(
            div()
                .flex()
                .flex_col()
                .mb_6()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_2xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("Calix Settings"),
                        )
                        .child(if is_dirty {
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.warning)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.bg_app)
                                .child("Unsaved Changes")
                        } else {
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme.bg_surface)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.text_muted)
                                .child("Up to Date")
                        }),
                )
                .child(div().text_xs().text_color(theme.text_muted).child(
                    "Configuration of package sources, desktop preferences, and motion policy.",
                )),
        );

        // Section 1: PACKAGE SOURCES
        let sources_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("PACKAGE SOURCES"),
            )
            .child(Self::toggle_row(
                "setting_aur",
                "Arch User Repository (AUR)",
                "Enable search and PKGBUILD inspection for community AUR packages",
                s.aur_enabled,
                props.on_toggle_aur,
                theme,
            ))
            .child(Self::toggle_row(
                "setting_flatpak",
                "Flatpak Packages",
                "Enable search and inspection for sandboxed applications via Flathub",
                s.flat_pack_enabled,
                props.on_toggle_flatpak,
                theme,
            ))
            .child(Self::toggle_row(
                "setting_appimage",
                "AppImage Packages",
                "Enable discovery and inspection for self-contained AppImage binaries",
                s.app_image_enabled,
                props.on_toggle_appimage,
                theme,
            ));

        root = root.child(sources_section);

        // Section 2: PACKAGE MANAGEMENT
        let mgmt_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("PACKAGE MANAGEMENT"),
            )
            .child(Self::toggle_row(
                "setting_cascade_delete",
                "Cascade Dependency Removal (--cascade)",
                "Recursively remove orphaned dependencies when removing packages",
                s.package_management_cascade_delete,
                props.on_toggle_cascade_delete,
                theme,
            ))
            .child(Self::toggle_row(
                "setting_remove_configs",
                "Remove Configuration Files (--remove-config)",
                "Purge package configuration and backup files upon package removal",
                s.package_management_remove_configs,
                props.on_toggle_remove_configs,
                theme,
            ));

        root = root.child(mgmt_section);

        // Section 3: APPEARANCE & DENSITY
        let appearance_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("APPEARANCE & DENSITY"),
            )
            .child(Self::toggle_row(
                "setting_dark_theme",
                "Dark Theme",
                "Use high-contrast dark theme interface (uncheck for light theme)",
                g.dark_theme,
                props.on_toggle_dark_theme,
                theme,
            ))
            .child(Self::brand_row(
                g.brand_color,
                props.on_select_brand.clone(),
                theme,
            ))
            .child(Self::toggle_row(
                "setting_compact_view",
                "Compact View Density",
                "Reduce card and table row heights and collapse the navigation sidebar",
                g.compact_view,
                props.on_toggle_compact_view,
                theme,
            ))
            .child(Self::style_select_row(StyleSelectRowProps {
                current: g.visual_style,
                menu_open: props.style_menu_open,
                menu_epoch: props.style_menu_epoch,
                menu_highlighted: props.style_menu_highlighted,
                reduce_motion: g.reduce_motion,
                on_open: props.on_open_style_menu,
                on_close: props.on_close_style_menu,
                on_navigate: props.on_navigate_style_menu,
                on_select: props.on_select_style,
                theme,
            }))
            .child(
                div()
                    .px_3()
                    .pt_2()
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child(
                        "Transparency is declared in STYLE-00A; its visual projection arrives with STYLE-00B after Render Lab evidence.",
                    ),
            );

        root = root.child(appearance_section);

        // Section 4: MOTION & FEEDBACK
        let motion_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("MOTION & FEEDBACK"),
            )
            .child(Self::toggle_row(
                "setting_reduce_motion",
                "Reduce Motion",
                "Disable animated transitions and snap sidebar, console, and toasts immediately",
                g.reduce_motion,
                props.on_toggle_reduce_motion,
                theme,
            ));

        root = root.child(motion_section);

        // Section 5: LOGS & OPERATIONS
        let logs_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent)
                    .mb_3()
                    .child("LOGS & OPERATIONS"),
            )
            .child(Self::toggle_row(
                "setting_log_drawer_auto_open",
                "Auto-Open Operation Console",
                "Automatically reveal the log drawer whenever a package mutation starts",
                g.log_drawer_open,
                props.on_toggle_log_drawer_auto_open,
                theme,
            ));

        root = root.child(logs_section);

        let advanced_toggle = props.on_toggle_advanced_settings.clone();
        let advanced_toggle_key = props.on_toggle_advanced_settings.clone();
        let advanced_open = props.on_open_render_lab.clone();
        let advanced_open_key = props.on_open_render_lab;
        let focus_border = theme.border_focus;
        let advanced_section = div()
            .flex()
            .flex_col()
            .p_4()
            .mb_6()
            .rounded_md()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(theme.border)
            .child(
                div()
                    .id("settings_advanced_toggle")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            advanced_toggle_key(window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        advanced_toggle(window, cx);
                    })
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.accent)
                            .child("ADVANCED SETTINGS"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.text_secondary)
                            .child(if props.advanced_settings_expanded {
                                "Collapse"
                            } else {
                                "Expand"
                            })
                            .child(if props.advanced_settings_expanded {
                                "  ▴"
                            } else {
                                "  ▾"
                            }),
                    ),
            )
            .when(props.advanced_settings_expanded, |section| {
                section.child(
                    div()
                        .mt_3()
                        .p_3()
                        .rounded_md()
                        .bg(theme.bg_app)
                        .border_1()
                        .border_color(theme.border)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    svg()
                                        .path(AppIcon::RenderLab.path())
                                        .size(px(16.0))
                                        .text_color(theme.accent),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.text_primary)
                                                .child("Render Lab"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme.text_secondary)
                                                .child("Open the experimental rendering and visual capability laboratory."),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .id("settings_render_lab_btn")
                                .focusable()
                                .tab_stop(true)
                                .focus(move |s| s.border_1().border_color(theme.border_focus))
                                .on_key_down(move |event, window, cx| {
                                    let key = event.keystroke.key.as_str();
                                    if key == "enter" || key == "space" {
                                        advanced_open_key(window, cx);
                                    }
                                })
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(theme.bg_surface_active)
                                .border_1()
                                .border_color(theme.border)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .cursor_pointer()
                                .hover(move |s| s.bg(theme.bg_surface_hover))
                                .child("Open")
                                .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                                    advanced_open(window, cx);
                                }),
                        ),
                )
            });

        root = root.child(advanced_section);

        // Bouton de sauvegarde explicite & bouton de réinitialisation
        let on_save = props.on_save;
        let on_reset = props.on_reset;
        let mut actions_bar = div().flex().items_center().justify_end().gap_3().mt_6();

        if is_dirty {
            if let Some(reset_handler) = on_reset {
                let reset_key = reset_handler.clone();
                let focus_border = theme.border_focus;
                actions_bar = actions_bar.child(
                    div()
                        .id("reset_settings_btn")
                        .focusable()
                        .tab_stop(true)
                        .focus(move |s| s.border_1().border_color(focus_border))
                        .on_key_down(move |event, window, cx| {
                            let key = event.keystroke.key.as_str();
                            if key == "enter" || key == "space" {
                                reset_key(window, cx);
                            }
                        })
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .bg(theme.bg_surface)
                        .border_1()
                        .border_color(theme.border)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.text_secondary)
                        .cursor_pointer()
                        .hover(move |s| s.bg(theme.bg_surface_hover).text_color(theme.text_primary))
                        .child("Reset Changes")
                        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                            reset_handler(window, cx);
                        }),
                );
            }

            let on_save_key = on_save.clone();
            let focus_border = theme.border_focus;
            actions_bar = actions_bar.child(
                div()
                    .id("save_settings_btn")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_save_key(window, cx);
                        }
                    })
                    .px_6()
                    .py_2()
                    .rounded_md()
                    .bg(theme.accent)
                    .border_1()
                    .border_color(theme.accent)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.bg_app)
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.accent_hover).text_color(theme.bg_app))
                    .child("Save Settings *")
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        on_save(window, cx);
                    }),
            );
        } else {
            actions_bar = actions_bar.child(
                div()
                    .id("save_settings_btn")
                    .px_6()
                    .py_2()
                    .rounded_md()
                    .bg(theme.bg_surface_active)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_muted)
                    .child("Settings Saved"),
            );
        }

        root = root.child(actions_bar);
        root
    }

    fn style_select_row(props: StyleSelectRowProps) -> impl IntoElement {
        let theme = props.theme;
        let focus_border = theme.border_focus;
        let on_open_key = props.on_open.clone();
        let on_open = props.on_open;
        let on_close = props.on_close;
        let on_navigate = props.on_navigate;
        let on_select = props.on_select;
        let current = props.current;
        let menu_open = props.menu_open;
        let menu_epoch = props.menu_epoch;
        let menu_highlighted = props.menu_highlighted;
        let reduce_motion = props.reduce_motion;

        let mut items = Vec::new();
        for (idx, style) in VisualStyleId::ALL.iter().enumerate() {
            let select_cb = on_select.clone();
            let close_cb = on_close.clone();
            let style_val = *style;
            let id_str = match style {
                VisualStyleId::Standard => "menu_style_standard",
                VisualStyleId::Transparency => "menu_style_transparency",
            };
            items.push(
                MenuCheckmarkItem::render(
                    id_str.into(),
                    style.label(),
                    current == *style,
                    menu_open && menu_highlighted == idx,
                    theme,
                    Rc::new(move |w, a| {
                        select_cb(style_val, w, a);
                        close_cb(w, a);
                    }),
                )
                .into_any_element(),
            );
        }

        let dropdown = if menu_open {
            Some(
                deferred(
                    anchored()
                        .anchor(Corner::TopLeft)
                        .offset(point(px(0.0), px(34.0)))
                        .snap_to_window()
                        .child(MenuSurface::render(MenuSurfaceProps {
                            id: "style_menu_surface".into(),
                            theme,
                            min_width: px(220.0),
                            reduce_motion,
                            lifecycle: MenuLifecycle::Open,
                            anim_epoch: menu_epoch,
                            focus_handle: None,
                            visual_style: current,
                            on_close: on_close.clone(),
                            on_key_navigate: Some(on_navigate),
                            children: items,
                        })),
                )
                .into_any_element(),
            )
        } else {
            None
        };

        div()
            .flex()
            .items_center()
            .justify_between()
            .py_3()
            .px_3()
            .rounded_sm()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child("Visual Style"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary)
                            .child("Selectable projection profile: Standard or Transparency"),
                    ),
            )
            .child(
                div()
                    .id("setting_visual_style_btn")
                    .relative()
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_open_key(window, cx);
                        }
                    })
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(theme.bg_surface_active)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text_primary)
                    .cursor_pointer()
                    .hover(move |s| s.bg(theme.bg_surface_hover))
                    .child(current.label())
                    .child("▾")
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        on_open(window, cx);
                    })
                    .children(dropdown),
            )
    }

    /// La couleur de marque : deux pastilles, la couleur choisie.
    ///
    /// La pastille porte la couleur qu'elle choisit, et l'accent de l'interface
    /// suit le choix : la coupe en écalille de la barre latérale et l'interface
    /// parlent alors la même couleur.
    fn brand_row(
        current: Brand,
        on_select: BrandSelectHandler,
        theme: &Theme,
    ) -> AnyElement {
        let focus_border = theme.border_focus;
        let mut swatches: Vec<AnyElement> = Vec::new();
        for brand in Brand::ALL.iter() {
            let select_cb = on_select.clone();
            let select_cb_key = select_cb.clone();
            let brand_val = *brand;
            let selected = current == *brand;
            swatches.push(
                div()
                    .id(ElementId::Name(
                        format!("setting_brand_{}", brand.as_str()).into(),
                    ))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .border_1()
                    .border_color(if selected {
                        brand.accent()
                    } else {
                        theme.border
                    })
                    .bg(if selected {
                        theme.bg_surface_active
                    } else {
                        theme.bg_surface
                    })
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            select_cb_key(brand_val, window, cx);
                        }
                    })
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                        select_cb(brand_val, window, cx);
                    })
                    .child(div().size_3().rounded_full().bg(brand.accent()))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(if selected {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if selected {
                                theme.text_primary
                            } else {
                                theme.text_secondary
                            })
                            .child(brand.label()),
                    )
                    .into_any_element(),
            );
        }

        div()
            .flex()
            .items_center()
            .justify_between()
            .py_3()
            .px_3()
            .rounded_sm()
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child("Brand Colour"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary)
                            .child("The colour of the shell mark, and the accent of the interface"),
                    ),
            )
            .child(div().flex().items_center().gap_2().children(swatches))
            .into_any_element()
    }

    fn toggle_row(
        id: &'static str,
        title: &'static str,
        description: &'static str,
        active: bool,
        on_toggle: WindowActionHandler,
        theme: &Theme,
    ) -> impl IntoElement {
        let hover_bg = theme.bg_surface_hover;
        let focus_border = theme.border_focus;
        let on_toggle_key = on_toggle.clone();
        div()
            .flex()
            .items_center()
            .justify_between()
            .py_3()
            .px_3()
            .rounded_sm()
            .border_b_1()
            .border_color(theme.border)
            .hover(move |s| s.bg(hover_bg))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(theme.text_secondary)
                            .child(description),
                    ),
            )
            .child(
                div()
                    .id(id)
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            on_toggle_key(window, cx);
                        }
                    })
                    .px_3()
                    .py_1()
                    .rounded_full()
                    .bg(if active { theme.success } else { theme.border })
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .text_color(if active {
                        theme.bg_app
                    } else {
                        theme.text_muted
                    })
                    .child(if active { "Enabled" } else { "Disabled" })
                    .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                        on_toggle(window, cx);
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_brand_colour_is_a_draft_change_like_the_other_toggles() {
        let mut settings = SettingsView::new(CalixSettings::default(), GpuiUiConfig::default());
        assert!(!settings.is_dirty);
        assert_eq!(settings.draft_gpui.brand_color, Brand::default());

        settings.set_brand_color(Brand::Royal);
        assert_eq!(settings.draft_gpui.brand_color, Brand::Royal);
        assert!(
            settings.is_dirty,
            "the colour is unsaved until the user saves the settings"
        );
    }

    #[test]
    fn test_settings_draft_dirty_flag() {
        let mut settings = SettingsView::new(CalixSettings::default(), GpuiUiConfig::default());
        assert!(!settings.is_dirty);

        settings.toggle_aur();
        assert!(settings.is_dirty);
        assert!(!settings.draft_calix.aur_enabled);

        settings.toggle_flatpak();
        assert!(settings.is_dirty);
        assert!(!settings.draft_calix.flat_pack_enabled);

        settings.toggle_appimage();
        assert!(!settings.draft_calix.app_image_enabled);

        settings.toggle_cascade_delete();
        assert!(!settings.draft_calix.package_management_cascade_delete);

        settings.toggle_remove_configs();
        assert!(!settings.draft_calix.package_management_remove_configs);

        settings.toggle_dark_theme();
        assert!(!settings.draft_gpui.dark_theme);

        settings.toggle_compact_view();
        assert!(settings.draft_gpui.compact_view);

        settings.toggle_log_drawer_auto_open();
        assert!(settings.draft_gpui.log_drawer_open);
    }

    #[test]
    fn test_settings_save_clears_dirty_on_success() {
        let mut settings = SettingsView::new(CalixSettings::default(), GpuiUiConfig::default());
        settings.toggle_reduce_motion();
        assert!(settings.is_dirty);

        let result = settings.save_with(|_| Ok(()), |_| Ok(()));
        assert!(result.is_ok());
        assert!(!settings.is_dirty, "Successful save must clear dirty flag");
        assert!(settings.draft_gpui.reduce_motion);
    }

    #[test]
    fn test_settings_save_preserves_dirty_on_failure() {
        let mut settings = SettingsView::new(CalixSettings::default(), GpuiUiConfig::default());
        settings.toggle_aur();
        assert!(settings.is_dirty);

        let result = settings.save_with(|_| anyhow::bail!("Permission denied"), |_| Ok(()));
        assert!(result.is_err());
        assert!(
            settings.is_dirty,
            "Failed save must preserve dirty flag so draft is not lost"
        );
    }

    #[test]
    fn test_reduce_motion_toggle_updates_draft() {
        let mut settings = SettingsView::new(CalixSettings::default(), GpuiUiConfig::default());
        assert!(!settings.draft_gpui.reduce_motion);

        settings.toggle_reduce_motion();
        assert!(settings.draft_gpui.reduce_motion);
        assert!(settings.is_dirty);

        settings.toggle_reduce_motion();
        assert!(!settings.draft_gpui.reduce_motion);

        assert!(!settings.advanced_settings_expanded);
        settings.toggle_advanced_settings();
        assert!(settings.advanced_settings_expanded);
        settings.toggle_advanced_settings();
        assert!(!settings.advanced_settings_expanded);
    }

    #[test]
    fn test_settings_clean_state_and_draft_isolation() {
        let initial_calix = CalixSettings::default();
        let initial_gpui = GpuiUiConfig::default();
        let mut settings = SettingsView::new(initial_calix.clone(), initial_gpui.clone());
        assert!(!settings.is_dirty);

        // Toggling reduce motion makes it dirty
        settings.toggle_reduce_motion();
        assert!(settings.is_dirty);
        assert_ne!(
            settings.draft_gpui.reduce_motion,
            initial_gpui.reduce_motion
        );

        // Toggling back does not automatically clear is_dirty (draft was touched)
        settings.toggle_reduce_motion();
        assert!(settings.is_dirty);
        assert_eq!(
            settings.draft_gpui.reduce_motion,
            initial_gpui.reduce_motion
        );

        // Only explicit save clears is_dirty
        let result = settings.save_with(|_| Ok(()), |_| Ok(()));
        assert!(result.is_ok());
        assert!(!settings.is_dirty);
    }

    #[test]
    fn test_style_menu_open_close_navigate_select() {
        let mut settings = SettingsView::new(CalixSettings::default(), GpuiUiConfig::default());
        assert_eq!(settings.draft_gpui.visual_style, VisualStyleId::Standard);
        assert!(!settings.style_menu_open);

        settings.open_style_menu();
        assert!(settings.style_menu_open);
        assert_eq!(settings.style_menu_epoch, 1);
        assert_eq!(settings.style_menu_highlighted, 0);
        assert!(
            !settings.is_dirty,
            "Opening the menu must not dirty the draft"
        );

        settings.navigate_style_menu("down");
        assert_eq!(settings.style_menu_highlighted, 1);
        settings.navigate_style_menu("down");
        assert_eq!(settings.style_menu_highlighted, 0);
        settings.navigate_style_menu("up");
        assert_eq!(settings.style_menu_highlighted, 1);
        settings.navigate_style_menu("home");
        assert_eq!(settings.style_menu_highlighted, 0);
        settings.navigate_style_menu("end");
        assert_eq!(settings.style_menu_highlighted, 1);

        settings.navigate_style_menu("enter");
        assert_eq!(
            settings.draft_gpui.visual_style,
            VisualStyleId::Transparency
        );
        assert!(settings.is_dirty);
        assert!(!settings.style_menu_open);

        settings.open_style_menu();
        assert_eq!(settings.style_menu_highlighted, 1);
        settings.close_style_menu();
        assert!(!settings.style_menu_open);
    }

    #[test]
    fn test_set_visual_style_dirties_and_persists_through_save() {
        let mut settings = SettingsView::new(CalixSettings::default(), GpuiUiConfig::default());
        settings.set_visual_style(VisualStyleId::Transparency);
        assert!(settings.is_dirty);

        let mut saved_style = VisualStyleId::Standard;
        let result = settings.save_with(
            |_| Ok(()),
            |gpui| {
                saved_style = gpui.visual_style;
                Ok(())
            },
        );
        assert!(result.is_ok());
        assert!(!settings.is_dirty);
        assert_eq!(saved_style, VisualStyleId::Transparency);
    }

    #[test]
    fn test_settings_reset_restores_saved_state() {
        let initial_calix = CalixSettings::default();
        let initial_gpui = GpuiUiConfig::default();
        let mut settings = SettingsView::new(initial_calix.clone(), initial_gpui.clone());

        settings.toggle_aur();
        settings.toggle_dark_theme();
        assert!(settings.is_dirty);

        settings.reset_to(initial_calix.clone(), initial_gpui.clone());
        assert!(!settings.is_dirty);
        assert_eq!(
            settings.draft_calix.aur_enabled,
            initial_calix.aur_enabled
        );
        assert_eq!(settings.draft_gpui.dark_theme, initial_gpui.dark_theme);
    }
}
