use crate::backend::models::ArchNewsItem;
use crate::icons::AppIcon;
use crate::theme::Theme;
use gpui::*;
use std::rc::Rc;

pub type NewsRetryHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

pub struct NewsViewProps<'a> {
    pub news: &'a [ArchNewsItem],
    pub error: Option<&'a str>,
    pub theme: &'a Theme,
    pub is_loading: bool,
    pub on_retry: Option<NewsRetryHandler>,
}

pub struct NewsView;

impl NewsView {
    pub fn render(props: NewsViewProps) -> impl IntoElement {
        let theme = props.theme;

        let mut root = div()
            .id("news_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.bg_app)
            .p_6()
            .overflow_scroll();

        root = root.child(
            div()
                .flex()
                .flex_col()
                .mb_6()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("Official Arch Linux News"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_muted)
                        .child("Important announcements, package updates, and recommended manual interventions."),
                ),
        );

        if props.is_loading {
            return root.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(200.0))
                    .text_color(theme.text_muted)
                    .child("Loading Arch Linux news..."),
            );
        }

        if let Some(err) = props.error {
            let on_retry = props.on_retry.clone();
            return root.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .h(px(200.0))
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.danger)
                            .child(format!("Failed to load Arch Linux news: {err}")),
                    )
                    .children(on_retry.map(|cb| {
                        div()
                            .id("retry_news_btn")
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(theme.accent)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.bg_app)
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.accent_hover))
                            .child("Retry")
                            .on_mouse_down(MouseButton::Left, move |_ev, w, cx| {
                                cb(w, cx);
                            })
                    })),
            );
        }

        if props.news.is_empty() {
            return root.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(200.0))
                    .text_color(theme.text_muted)
                    .child("No recent announcements to display."),
            );
        }

        for (idx, item) in props.news.iter().enumerate() {
            let mut card = div()
                .id(("news_card", idx))
                .flex()
                .flex_col()
                .p_4()
                .mb_4()
                .rounded_md()
                .bg(theme.bg_surface)
                .border_1()
                .border_color(theme.border);

            if let Some(url) = item.url.clone() {
                let url_key = url.clone();
                let focus_border = theme.border_focus;
                card = card
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(focus_border))
                    .on_key_down(move |event, _window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            cx.open_url(&url_key);
                        }
                    })
                    .cursor_pointer()
                    .hover(|s| s.border_color(theme.accent))
                    .on_mouse_down(MouseButton::Left, move |_ev, _window, cx| {
                        cx.open_url(&url);
                    });
            }

            card = card
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .mb_2()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_base()
                                .text_color(theme.text_primary)
                                .child(item.title.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.accent)
                                .child(item.published_date.clone().unwrap_or_default()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.text_secondary)
                        .mb_2()
                        .child(if let Some(ref author) = item.author {
                            format!("Published by: {}", author)
                        } else {
                            "Arch Linux Team".to_string()
                        }),
                )
                .child(div().text_sm().text_color(theme.text_primary).child(
                    item.summary.clone().unwrap_or_else(|| {
                        "Click to read the full announcement on archlinux.org".to_string()
                    }),
                ));

            if item.url.is_some() {
                card = card.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .text_xs()
                        .text_color(theme.accent)
                        .mt_2()
                        .child(
                            svg()
                                .path(AppIcon::ExternalUrl.path())
                                .size_3()
                                .text_color(theme.accent),
                        )
                        .child("Open in browser"),
                );
            }

            root = root.child(card);
        }

        root
    }
}
