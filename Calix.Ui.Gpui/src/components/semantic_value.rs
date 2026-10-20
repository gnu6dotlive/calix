use crate::state::SemanticTarget;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type StringActionHandler = Rc<dyn Fn(String, &mut Window, &mut App) + 'static>;

pub struct SemanticValue;

impl SemanticValue {
    /// Rendu d'une valeur sémantique interactive (URL, chemin de fichier, nom de paquet ou texte copiable)
    pub fn render(
        display_text: String,
        target: SemanticTarget,
        theme: &Theme,
        on_navigate_package: Option<StringActionHandler>,
    ) -> impl IntoElement {
        let hover_color = theme.accent;
        let text_color = theme.text_primary;

        let icon = match &target {
            SemanticTarget::ExternalUrl(_) => crate::icons::AppIcon::ExternalUrl,
            SemanticTarget::FilePath(_) => crate::icons::AppIcon::FilePath,
            SemanticTarget::Package(_) => crate::icons::AppIcon::Installed,
            SemanticTarget::CopyText(_) => crate::icons::AppIcon::Copy,
        };

        let target_click = target;

        div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .cursor_pointer()
            .text_sm()
            .text_color(text_color)
            .hover(move |s| s.text_color(hover_color).underline())
            .child(
                svg()
                    .path(icon.path())
                    .size_3()
                    .text_color(theme.text_muted),
            )
            .child(div().overflow_hidden().text_ellipsis().child(display_text))
            .on_mouse_down(
                MouseButton::Left,
                move |_event, window, cx| match &target_click {
                    SemanticTarget::ExternalUrl(url) => {
                        cx.open_url(url);
                    }
                    SemanticTarget::FilePath(path) | SemanticTarget::CopyText(path) => {
                        cx.write_to_clipboard(ClipboardItem::new_string(path.clone()));
                    }
                    SemanticTarget::Package(pkg_name) => {
                        if let Some(ref nav) = on_navigate_package {
                            nav(pkg_name.clone(), window, cx);
                        }
                    }
                },
            )
    }
}
