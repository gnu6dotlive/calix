use crate::icons::AppIcon;
use crate::theme::Theme;
use gpui::*;
use std::ops::Range;

actions!(
    search_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Paste,
        Cut,
        Copy,
        Escape,
        Enter,
    ]
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchEvent {
    Changed(String),
    Submitted(String),
    Cleared,
}

pub struct SearchInputView {
    pub focus_handle: FocusHandle,
    pub content: SharedString,
    pub placeholder: SharedString,
    pub selected_range: Range<usize>,
    pub selection_reversed: bool,
    pub marked_range: Option<Range<usize>>,
    pub last_layout: Option<ShapedLine>,
    pub last_bounds: Option<Bounds<Pixels>>,
    pub is_selecting: bool,
    pub is_searching: bool,
    pub reduce_motion: bool,
    pub theme: Theme,
}

impl EventEmitter<SearchEvent> for SearchInputView {}

impl Focusable for SearchInputView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl SearchInputView {
    pub fn new(
        placeholder: impl Into<SharedString>,
        theme: Theme,
        reduce_motion: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: placeholder.into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            is_searching: false,
            reduce_motion,
            theme,
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        self.selected_range = self.content.len()..self.content.len();
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    pub fn set_is_searching(&mut self, is_searching: bool, cx: &mut Context<Self>) {
        if self.is_searching != is_searching {
            self.is_searching = is_searching;
            cx.notify();
        }
    }

    pub fn set_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        self.theme = theme;
        cx.notify();
    }

    pub fn focus(&self, window: &mut Window) {
        window.focus(&self.focus_handle);
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    pub fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    pub fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .char_indices()
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }

    pub fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    pub fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    pub fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    pub fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    pub fn replace_text_in_range_internal(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        let start = range.start.min(self.content.len());
        let end = range.end.min(self.content.len());

        let mut next = String::with_capacity(self.content.len() - (end - start) + new_text.len());
        next.push_str(&self.content[..start]);
        next.push_str(new_text);
        next.push_str(&self.content[end..]);

        let new_cursor = start + new_text.len();
        self.content = next.into();
        self.selected_range = new_cursor..new_cursor;
        self.selection_reversed = false;
        self.marked_range = None;
    }

    pub fn move_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
    }

    pub fn select_to(&mut self, offset: usize) {
        let offset = offset.min(self.content.len());
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
    }

    pub fn select_all_internal(&mut self) {
        self.selected_range = 0..self.content.len();
        self.selection_reversed = false;
    }

    pub fn backspace_internal(&mut self) {
        if self.selected_range.is_empty() {
            let prev = self.previous_boundary(self.cursor_offset());
            self.selected_range = prev..self.cursor_offset();
        }
        self.replace_text_in_range_internal(None, "");
    }

    pub fn delete_internal(&mut self) {
        if self.selected_range.is_empty() {
            let next = self.next_boundary(self.cursor_offset());
            self.selected_range = self.cursor_offset()..next;
        }
        self.replace_text_in_range_internal(None, "");
    }

    pub fn clear_internal(&mut self) {
        self.content = "".into();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
    }

    pub fn clear_and_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_internal();
        self.focus(window);
        cx.emit(SearchEvent::Cleared);
        cx.emit(SearchEvent::Changed(String::new()));
        cx.notify();
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()));
        } else {
            self.move_to(self.selected_range.start);
        }
        cx.notify();
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end));
        } else {
            self.move_to(self.selected_range.end);
        }
        cx.notify();
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()));
        cx.notify();
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()));
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.select_all_internal();
        cx.notify();
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0);
        cx.notify();
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len());
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        self.backspace_internal();
        cx.emit(SearchEvent::Changed(self.content.to_string()));
        cx.notify();
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        self.delete_internal();
        cx.emit(SearchEvent::Changed(self.content.to_string()));
        cx.notify();
    }

    fn escape(&mut self, _: &Escape, window: &mut Window, cx: &mut Context<Self>) {
        self.clear_internal();
        self.focus(window);
        cx.emit(SearchEvent::Cleared);
        cx.emit(SearchEvent::Changed(String::new()));
        cx.notify();
    }

    fn enter(&mut self, _: &Enter, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(SearchEvent::Submitted(self.content.to_string()));
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            let sanitized = text.replace(['\r', '\n'], " ");
            self.replace_text_in_range_internal(None, &sanitized);
            cx.emit(SearchEvent::Changed(self.content.to_string()));
            cx.notify();
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            let start = self.selected_range.start.min(self.content.len());
            let end = self.selected_range.end.min(self.content.len());
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[start..end].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            let start = self.selected_range.start.min(self.content.len());
            let end = self.selected_range.end.min(self.content.len());
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[start..end].to_string(),
            ));
            self.replace_text_in_range_internal(None, "");
            cx.emit(SearchEvent::Changed(self.content.to_string()));
            cx.notify();
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus(window);
        self.is_selecting = true;
        let idx = self.index_for_mouse_position(event.position);
        if event.modifiers.shift {
            self.select_to(idx);
        } else {
            self.move_to(idx);
        }
        cx.notify();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_selecting = false;
        cx.notify();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            let idx = self.index_for_mouse_position(event.position);
            self.select_to(idx);
            cx.notify();
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }
}

impl EntityInputHandler for SearchInputView {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        let start = range.start.min(self.content.len());
        let end = range.end.min(self.content.len());
        actual_range.replace(self.range_to_utf16(&(start..end)));
        Some(self.content[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_text_in_range_internal(range_utf16, new_text);
        cx.emit(SearchEvent::Changed(self.content.to_string()));
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        let start = range.start.min(self.content.len());
        let end = range.end.min(self.content.len());

        let mut next = String::with_capacity(self.content.len() - (end - start) + new_text.len());
        next.push_str(&self.content[..start]);
        next.push_str(new_text);
        next.push_str(&self.content[end..]);

        if !new_text.is_empty() {
            self.marked_range = Some(start..start + new_text.len());
        } else {
            self.marked_range = None;
        }

        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .map(|new_range| new_range.start + start..new_range.end + start)
            .unwrap_or_else(|| start + new_text.len()..start + new_text.len());

        self.content = next.into();
        cx.emit(SearchEvent::Changed(self.content.to_string()));
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let start = range.start.min(self.content.len());
        let end = range.end.min(self.content.len());
        Some(Bounds::from_corners(
            point(bounds.left() + last_layout.x_for_index(start), bounds.top()),
            point(
                bounds.left() + last_layout.x_for_index(end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        if last_layout.text != self.content {
            return None;
        }
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

pub struct SearchInputElement {
    input: Entity<SearchInputView>,
}

pub struct SearchInputPrepaint {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for SearchInputElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SearchInputElement {
    type RequestLayoutState = ();
    type PrepaintState = SearchInputPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();
        let theme = &input.theme;

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), theme.text_muted)
        } else {
            (content, theme.text_primary)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color.into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor_quad) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(2.), bounds.bottom() - bounds.top()),
                    ),
                    theme.accent,
                )),
            )
        } else {
            let start_x = line.x_for_index(selected_range.start);
            let end_x = line.x_for_index(selected_range.end);
            let selection_color = Rgba {
                a: 0.25,
                ..theme.accent
            };
            (
                Some(fill(
                    Bounds::from_corners(
                        point(bounds.left() + start_x, bounds.top()),
                        point(bounds.left() + end_x, bounds.bottom()),
                    ),
                    selection_color,
                )),
                None,
            )
        };

        SearchInputPrepaint {
            line: Some(line),
            cursor: cursor_quad,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let line = prepaint.line.take().unwrap();
        let _ = line.paint(bounds.origin, window.line_height(), window, cx);

        if focus_handle.is_focused(window) {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for SearchInputView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_focused = self.focus_handle.is_focused(window);
        let theme = self.theme;
        let border_color = if is_focused {
            theme.border_focus
        } else {
            theme.border
        };

        let has_content = !self.content.is_empty();
        let is_searching = self.is_searching;
        let reduce_motion = self.reduce_motion;

        let clear_button = if has_content {
            Some(
                div()
                    .id("search_clear_button")
                    .focusable()
                    .tab_stop(true)
                    .focus(move |s| s.border_1().border_color(theme.border_focus))
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        let key = event.keystroke.key.as_str();
                        if key == "enter" || key == "space" {
                            this.clear_and_focus(window, cx);
                        }
                    }))
                    .flex_none()
                    .cursor_pointer()
                    .p(px(4.0))
                    .rounded_sm()
                    .hover(|s| s.bg(theme.bg_surface_hover))
                    .child(
                        svg()
                            .path(AppIcon::Close.path())
                            .size(px(14.0))
                            .text_color(theme.text_muted),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _event, window, cx| {
                            this.clear_and_focus(window, cx);
                        }),
                    ),
            )
        } else {
            None
        };

        let search_indicator = if is_searching {
            let indicator_dot = div().size(px(7.0)).rounded_full().bg(theme.accent);

            let indicator_el = if reduce_motion {
                indicator_dot.into_any_element()
            } else {
                indicator_dot
                    .with_animation(
                        ("search_pulse", 0usize),
                        Animation::new(std::time::Duration::from_millis(700))
                            .repeat()
                            .with_easing(gpui::ease_in_out),
                        |el, delta| {
                            let opacity = 0.25 + 0.75 * (delta * std::f32::consts::PI).sin().abs();
                            el.opacity(opacity)
                        },
                    )
                    .into_any_element()
            };

            Some(
                div()
                    .id("searching_indicator")
                    .flex_none()
                    .size(px(14.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(indicator_el)
                    .into_any_element(),
            )
        } else if !has_content {
            Some(
                div()
                    .flex_none()
                    .px(px(5.0))
                    .py(px(1.0))
                    .rounded_xs()
                    .bg(theme.bg_surface)
                    .border_1()
                    .border_color(theme.border)
                    .text_xs()
                    .text_color(theme.text_muted)
                    .child("/")
                    .into_any_element(),
            )
        } else {
            None
        };

        let mut container = div()
            .id("search_input_container")
            .key_context("SearchInput")
            .track_focus(&self.focus_handle)
            .w_full()
            .h_full()
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .bg(theme.bg_surface)
            .border_1()
            .border_color(border_color)
            .rounded_md();

        if is_focused {
            container = container.shadow_sm();
        }

        let container = container
            .hover(move |s| {
                if is_focused {
                    s
                } else {
                    s.border_color(theme.text_muted)
                }
            })
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::escape))
            .on_action(cx.listener(Self::enter))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_key_down(cx.listener(|_this, event: &KeyDownEvent, window, _cx| {
                let key = event.keystroke.key.as_str();
                if key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev();
                    } else {
                        window.focus_next();
                    }
                }
            }))
            .child(
                svg()
                    .path(AppIcon::Search.path())
                    .size(px(16.0))
                    .flex_none()
                    .text_color(if is_focused || is_searching {
                        theme.accent
                    } else {
                        theme.text_muted
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(SearchInputElement { input: cx.entity() }),
            )
            .children(search_indicator)
            .children(clear_button);

        if !reduce_motion {
            container
                .with_animation(
                    (
                        "search_focus_anim",
                        if is_focused { 1usize } else { 0usize },
                    ),
                    Animation::new(std::time::Duration::from_millis(110))
                        .with_easing(gpui::ease_out_quint()),
                    move |el, delta| {
                        if is_focused {
                            el.opacity(0.9 + 0.1 * delta)
                        } else {
                            el.opacity(1.0)
                        }
                    },
                )
                .into_any_element()
        } else {
            container.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    struct TestBuffer {
        content: SharedString,
        selected_range: Range<usize>,
        selection_reversed: bool,
        marked_range: Option<Range<usize>>,
    }

    impl TestBuffer {
        fn new() -> Self {
            Self {
                content: "".into(),
                selected_range: 0..0,
                selection_reversed: false,
                marked_range: None,
            }
        }

        fn cursor_offset(&self) -> usize {
            if self.selection_reversed {
                self.selected_range.start
            } else {
                self.selected_range.end
            }
        }

        fn previous_boundary(&self, offset: usize) -> usize {
            self.content
                .char_indices()
                .rev()
                .find_map(|(idx, _)| (idx < offset).then_some(idx))
                .unwrap_or(0)
        }

        fn next_boundary(&self, offset: usize) -> usize {
            self.content
                .char_indices()
                .find_map(|(idx, _)| (idx > offset).then_some(idx))
                .unwrap_or(self.content.len())
        }

        fn replace_text(&mut self, new_text: &str) {
            let range = self
                .marked_range
                .clone()
                .unwrap_or(self.selected_range.clone());
            let start = range.start.min(self.content.len());
            let end = range.end.min(self.content.len());

            let mut next =
                String::with_capacity(self.content.len() - (end - start) + new_text.len());
            next.push_str(&self.content[..start]);
            next.push_str(new_text);
            next.push_str(&self.content[end..]);

            let new_cursor = start + new_text.len();
            self.content = next.into();
            self.selected_range = new_cursor..new_cursor;
            self.selection_reversed = false;
            self.marked_range = None;
        }

        fn backspace(&mut self) {
            if self.selected_range.is_empty() {
                let prev = self.previous_boundary(self.cursor_offset());
                self.selected_range = prev..self.cursor_offset();
            }
            self.replace_text("");
        }

        fn delete(&mut self) {
            if self.selected_range.is_empty() {
                let next = self.next_boundary(self.cursor_offset());
                self.selected_range = self.cursor_offset()..next;
            }
            self.replace_text("");
        }

        fn select_all(&mut self) {
            self.selected_range = 0..self.content.len();
            self.selection_reversed = false;
        }

        fn clear(&mut self) {
            self.content = "".into();
            self.selected_range = 0..0;
            self.selection_reversed = false;
            self.marked_range = None;
        }
    }

    #[test]
    fn test_search_input_insert_and_caret_advance() {
        let mut buf = TestBuffer::new();
        buf.replace_text("ripgrep");
        assert_eq!(buf.content.as_ref(), "ripgrep");
        assert_eq!(buf.selected_range, 7..7);
        assert_eq!(buf.cursor_offset(), 7);

        buf.replace_text(" 14.1");
        assert_eq!(buf.content.as_ref(), "ripgrep 14.1");
        assert_eq!(buf.selected_range, 12..12);
    }

    #[test]
    fn test_search_input_backspace_and_delete() {
        let mut buf = TestBuffer::new();
        buf.replace_text("hello");
        buf.backspace();
        assert_eq!(buf.content.as_ref(), "hell");
        assert_eq!(buf.cursor_offset(), 4);

        // Move caret to index 1 ('e')
        buf.selected_range = 1..1;
        buf.delete();
        assert_eq!(buf.content.as_ref(), "hll");
        assert_eq!(buf.cursor_offset(), 1);
    }

    #[test]
    fn test_search_input_selection_replacement() {
        let mut buf = TestBuffer::new();
        buf.replace_text("the quick brown fox");
        // Select "quick " (range 4..10)
        buf.selected_range = 4..10;
        buf.replace_text("fast ");
        assert_eq!(buf.content.as_ref(), "the fast brown fox");
        assert_eq!(buf.selected_range, 9..9);
    }

    #[test]
    fn test_search_input_select_all_and_clear() {
        let mut buf = TestBuffer::new();
        buf.replace_text("package-query");
        buf.select_all();
        assert_eq!(buf.selected_range, 0..13);
        buf.replace_text("");
        assert_eq!(buf.content.as_ref(), "");
        assert_eq!(buf.selected_range, 0..0);

        buf.replace_text("another-query");
        buf.clear();
        assert_eq!(buf.content.as_ref(), "");
        assert_eq!(buf.selected_range, 0..0);
    }

    #[test]
    fn test_search_input_character_boundaries_utf8() {
        let mut buf = TestBuffer::new();
        buf.replace_text("café");
        assert_eq!(buf.content.len(), 5); // 'é' is 2 bytes in UTF-8
        assert_eq!(buf.cursor_offset(), 5);

        let prev = buf.previous_boundary(5);
        assert_eq!(prev, 3); // 'é' starts at byte 3
        buf.backspace();
        assert_eq!(buf.content.as_ref(), "caf");
        assert_eq!(buf.cursor_offset(), 3);
    }
}
