pub mod buffer;

use std::ops::Range;

use gpui::{
    App, Bounds, ClipboardItem, Context, CursorStyle, DispatchPhase, Element, ElementId,
    ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    GlobalElementId, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, Pixels, Point, ShapedLine, SharedString, Style, TextRun,
    UTF16Selection, Window, actions, div, fill, point, prelude::*, px, relative, size,
};

use buffer::{Buffer, Motion};
use zeroize::Zeroizing;

actions!(
    field,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Home,
        End,
        SelectLeft,
        SelectRight,
        SelectHome,
        SelectEnd,
        SelectAll,
        Paste,
        Copy,
        Cut
    ]
);

const CONTEXT: &str = "Field";
const CURSOR_WIDTH_PX: f32 = 2.0;
const SELECTION_OPACITY: f32 = 0.3;
const PLACEHOLDER_OPACITY: f32 = 0.45;
const WORD_CLICKS: usize = 2;
const BULLET: char = '•';

pub struct Changed;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mask {
    Plain,
    Secret,
}

pub struct Field {
    focus: FocusHandle,
    buffer: Buffer,
    mask: Mask,
    placeholder: SharedString,
    marked: Option<Range<usize>>,
    layout: Option<ShapedLine>,
    bounds: Option<Bounds<Pixels>>,
    dragging: bool,
}

struct Line {
    field: Entity<Field>,
}

struct Prepaint {
    line: ShapedLine,
    selection: Option<PaintQuad>,
    cursor: PaintQuad,
}

pub fn bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("backspace", Backspace, context),
        KeyBinding::new("delete", Delete, context),
        KeyBinding::new("left", Left, context),
        KeyBinding::new("right", Right, context),
        KeyBinding::new("home", Home, context),
        KeyBinding::new("end", End, context),
        KeyBinding::new("shift-left", SelectLeft, context),
        KeyBinding::new("shift-right", SelectRight, context),
        KeyBinding::new("shift-home", SelectHome, context),
        KeyBinding::new("shift-end", SelectEnd, context),
        KeyBinding::new("ctrl-a", SelectAll, context),
        KeyBinding::new("ctrl-v", Paste, context),
        KeyBinding::new("ctrl-c", Copy, context),
        KeyBinding::new("ctrl-x", Cut, context),
    ]
}

fn utf16_len(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

impl EventEmitter<Changed> for Field {}

impl Field {
    pub fn new(placeholder: &str, content: &str, cx: &mut Context<Self>) -> Self {
        Self::masked(placeholder, content, Mask::Plain, cx)
    }

    pub fn secret(placeholder: &str, cx: &mut Context<Self>) -> Self {
        Self::masked(placeholder, "", Mask::Secret, cx)
    }

    fn masked(placeholder: &str, content: &str, mask: Mask, cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            buffer: Buffer::new(content),
            mask,
            placeholder: placeholder.to_owned().into(),
            marked: None,
            layout: None,
            bounds: None,
            dragging: false,
        }
    }

    pub fn with_all_selected(mut self) -> Self {
        self.buffer.select_all();
        self
    }

    pub fn text(&self) -> &str {
        self.buffer.text()
    }

    pub fn set(&mut self, text: &str, cx: &mut Context<Self>) {
        let whole = 0..self.buffer.text().len();
        self.edit(|buffer| buffer.replace(whole, text), cx);
    }

    pub fn take(&mut self, cx: &mut Context<Self>) -> Zeroizing<String> {
        let taken = self.buffer.take();
        self.marked = None;
        cx.emit(Changed);
        cx.notify();
        taken
    }

    fn shown(&self) -> SharedString {
        let content = self.buffer.text();
        match self.mask {
            Mask::Plain => content.to_owned().into(),
            Mask::Secret => BULLET.to_string().repeat(content.chars().count()).into(),
        }
    }

    fn shown_offset(&self, offset: usize) -> usize {
        match self.mask {
            Mask::Plain => offset,
            Mask::Secret => self.buffer.text()[..offset].chars().count() * BULLET.len_utf8(),
        }
    }

    fn content_offset(&self, shown: usize) -> usize {
        let content = self.buffer.text();
        match self.mask {
            Mask::Plain => shown,
            Mask::Secret => content
                .char_indices()
                .nth(shown / BULLET.len_utf8())
                .map_or(content.len(), |(index, _)| index),
        }
    }

    fn edit(&mut self, change: impl FnOnce(&mut Buffer), cx: &mut Context<Self>) {
        change(&mut self.buffer);
        self.marked = None;
        cx.emit(Changed);
        cx.notify();
    }

    fn select(&mut self, change: impl FnOnce(&mut Buffer), cx: &mut Context<Self>) {
        change(&mut self.buffer);
        cx.notify();
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.move_by(Motion::Left), cx);
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.move_by(Motion::Right), cx);
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.move_by(Motion::Home), cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.move_by(Motion::End), cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.select_by(Motion::Left), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.select_by(Motion::Right), cx);
    }

    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.select_by(Motion::Home), cx);
    }

    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.select(|buffer| buffer.select_by(Motion::End), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.select(Buffer::select_all, cx);
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        self.edit(Buffer::backspace, cx);
    }

    fn delete(&mut self, _: &Delete, _: &mut Window, cx: &mut Context<Self>) {
        self.edit(Buffer::delete, cx);
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let line = text.replace(['\r', '\n'], " ");
        self.edit(|buffer| buffer.insert(&line), cx);
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        self.copy_selection(cx);
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if self.copy_selection(cx) {
            self.edit(|buffer| buffer.insert(""), cx);
        }
    }

    fn copy_selection(&self, cx: &mut Context<Self>) -> bool {
        let selected = self.buffer.selected();
        if selected.is_empty() || self.mask == Mask::Secret {
            return false;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(selected.to_owned()));
        true
    }

    fn press(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        self.focus.focus(window);
        let offset = self.offset_for_x(event.position);
        self.dragging = event.click_count < WORD_CLICKS;
        self.select(
            |buffer| match event.click_count {
                1 if event.modifiers.shift => buffer.select_to(offset),
                1 => buffer.move_to(offset),
                WORD_CLICKS => buffer.select_word_at(offset),
                _ => buffer.select_all(),
            },
            cx,
        );
    }

    fn drag(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if !self.dragging || !event.dragging() {
            return;
        }
        let offset = self.offset_for_x(event.position);
        self.select(|buffer| buffer.select_to(offset), cx);
    }

    fn release(&mut self) {
        self.dragging = false;
    }

    fn x_for(&self, offset: usize) -> Pixels {
        self.layout.as_ref().map_or(px(0.), |layout| {
            layout.x_for_index(self.shown_offset(offset))
        })
    }

    fn offset_for_x(&self, position: Point<Pixels>) -> usize {
        let (Some(layout), Some(bounds)) = (self.layout.as_ref(), self.bounds) else {
            return 0;
        };
        self.content_offset(layout.closest_index_for_x(position.x - bounds.left()))
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let content = self.buffer.text();
        let mut utf16 = 0;
        content
            .char_indices()
            .find(|(_, character)| {
                let reached = utf16 >= offset;
                utf16 += character.len_utf16();
                reached
            })
            .map_or(content.len(), |(index, _)| index)
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        utf16_len(&self.buffer.text()[..offset])
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range.start)..self.offset_from_utf16(range.end)
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn target_range(&self, range_utf16: Option<Range<usize>>) -> Range<usize> {
        range_utf16
            .map(|range| self.range_from_utf16(&range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.buffer.selection())
    }
}

impl EntityInputHandler for Field {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        adjusted_range.replace(self.range_to_utf16(&range));
        Some(self.buffer.text()[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.buffer.selection()),
            reversed: self.buffer.reversed(),
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked.as_ref().map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.target_range(range_utf16);
        self.edit(|buffer| buffer.replace(range, text), cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.target_range(range_utf16);
        let start = range.start;
        self.edit(|buffer| buffer.replace(range, text), cx);
        self.marked = (!text.is_empty()).then(|| start..start + text.len());
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(bounds.left() + self.x_for(range.start), bounds.top()),
            point(bounds.left() + self.x_for(range.end), bounds.bottom()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        position: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.offset_to_utf16(self.offset_for_x(position)))
    }
}

impl Line {
    fn track_drag(&self, window: &mut Window) {
        let moved = self.field.clone();
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, _, cx| {
            if phase == DispatchPhase::Bubble {
                moved.update(cx, |field, cx| field.drag(event, cx));
            }
        });
        let released = self.field.clone();
        window.on_mouse_event(move |_: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Bubble {
                released.update(cx, |field, _| field.release());
            }
        });
    }
}

impl IntoElement for Line {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Line {
    type RequestLayoutState = ();
    type PrepaintState = Prepaint;

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
        let field = self.field.read(cx);
        let content = field.buffer.text();
        let style = window.text_style();
        let (text, color) = match content.is_empty() {
            true => (
                field.placeholder.clone(),
                style.color.opacity(PLACEHOLDER_OPACITY),
            ),
            false => (field.shown(), style.color),
        };
        let run = TextRun {
            len: text.len(),
            font: style.font(),
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(text, font_size, &[run], None);
        let x = |offset: usize| bounds.left() + line.x_for_index(field.shown_offset(offset));
        let selected = field.buffer.selection();
        let selection = (!selected.is_empty()).then(|| {
            fill(
                Bounds::from_corners(
                    point(x(selected.start), bounds.top()),
                    point(x(selected.end), bounds.bottom()),
                ),
                style.color.opacity(SELECTION_OPACITY),
            )
        });
        let cursor = fill(
            Bounds::new(
                point(x(field.buffer.cursor()), bounds.top()),
                size(px(CURSOR_WIDTH_PX), bounds.size.height),
            ),
            style.color,
        );
        Prepaint {
            line,
            selection,
            cursor,
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
        let focus = self.field.read(cx).focus.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.field.clone()),
            cx,
        );
        self.track_drag(window);
        let focused = focus.is_focused(window);
        if let Some(selection) = prepaint.selection.take().filter(|_| focused) {
            window.paint_quad(selection);
        }
        let line = prepaint.line.clone();
        line.paint(bounds.origin, window.line_height(), window, cx)
            .ok();
        if focused {
            window.paint_quad(prepaint.cursor.clone());
        }
        self.field.update(cx, |field, _| {
            let shows_content = !field.buffer.text().is_empty();
            field.layout = shows_content.then_some(line);
            field.bounds = Some(bounds);
        });
    }
}

impl Focusable for Field {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for Field {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .w_full()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .cursor(CursorStyle::IBeam)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::press))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .child(Line { field: cx.entity() })
    }
}
