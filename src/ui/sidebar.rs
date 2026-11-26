use gpui::{
    AnyElement, Context, Div, ElementId, MouseButton, Stateful, Window, WindowControlArea, div,
    prelude::*, px, rgba,
};

use super::{
    NewTab,
    browser::Browser,
    context_menu::Target,
    drag::Zones,
    favicon::site_icon,
    frame::{self, Flow, INSET_PX},
    icon::Icon,
    session::pane_label,
    space::theme,
    surface::{CONTROL_CORNER_PX, Colors, ROW_HEIGHT_PX},
    tab::row_button,
    toolbar::{ADDRESS_HEIGHT_PX, BUTTON_PX, NAVIGATION_WIDTH_PX},
    top_bar::TOP_BAR_HEIGHT_PX,
};
use crate::{
    config::Switch,
    locale::Text,
    storage::{
        arrange::{Item, Spot},
        session::{Folder, Pinned, Section, Tab},
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Roomy,
    Compact,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HeaderRows {
    One,
    Two,
}

const ROOMY_MIN_PX: f32 = 110.0;
const ONE_ROW_HEADER_CONTENT_PX: f32 = BUTTON_PX + NAVIGATION_WIDTH_PX + 3.0 * frame::GAP_PX;
const TWO_ROW_HEADER_CONTENT_PX: f32 = BUTTON_PX + 2.0 * frame::GAP_PX;
const NAVIGATION_ROW_PX: f32 = BUTTON_PX;
const TAB_TITLE_SEPARATOR: &str = "  ⫼  ";
const FOLDER_INDENT_PX: f32 = 14.0;
const CLOSE_ICON_PX: f32 = 14.0;
const CHEVRON_PX: f32 = 12.0;
const DIVIDER_PX: f32 = 9.0;
const TAB_ROW_GROUP: &str = "tab-row";
const PROFILE_LABEL_MAX_PX: f32 = 72.0;
const PROFILE_LABEL_OPACITY: f32 = 0.7;

pub fn tab_title(tab: &Tab) -> String {
    if let Some(name) = tab.name() {
        return name.to_owned();
    }
    let titles: Vec<String> = tab.panes().iter().map(pane_label).collect();
    titles.join(TAB_TITLE_SEPARATOR)
}

fn divider_space() -> Div {
    div()
        .h(px(DIVIDER_PX))
        .px_1()
        .flex()
        .items_center()
        .window_control_area(WindowControlArea::Drag)
}

fn separator(colors: &Colors) -> Div {
    divider_space().child(div().flex_1().h(px(1.0)).bg(rgba(colors.line)))
}

fn sidebar_row(row: Stateful<Div>) -> Stateful<Div> {
    row.h(px(ROW_HEIGHT_PX))
        .flex()
        .items_center()
        .rounded(px(CONTROL_CORNER_PX))
}

fn roomy_row(row: Stateful<Div>, depth: usize) -> Stateful<Div> {
    sidebar_row(row)
        .px_2()
        .ml(px(FOLDER_INDENT_PX * depth as f32))
        .gap_2()
}

fn compact_row(row: Stateful<Div>) -> Stateful<Div> {
    sidebar_row(row).justify_center()
}

fn square_button(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(BUTTON_PX))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded(px(CONTROL_CORNER_PX))
}

impl Browser {
    fn header_fits(&self, content_px: f32) -> bool {
        let controls_px = frame::controls_width_px(self.layout().controls.style);
        self.sidebar_width_px() >= content_px + controls_px + 2.0 * INSET_PX
    }

    fn roomy_header_fits(&self) -> bool {
        match self.layout().top_bar {
            Switch::On => true,
            Switch::Off => self.header_fits(TWO_ROW_HEADER_CONTENT_PX),
        }
    }

    pub(super) fn density(&self) -> Density {
        match self.sidebar_width_px() >= ROOMY_MIN_PX && self.roomy_header_fits() {
            true => Density::Roomy,
            false => Density::Compact,
        }
    }

    fn header_rows(&self) -> HeaderRows {
        match self.header_fits(ONE_ROW_HEADER_CONTENT_PX) {
            true => HeaderRows::One,
            false => HeaderRows::Two,
        }
    }

    fn header_height_px(&self) -> f32 {
        match self.header_rows() {
            HeaderRows::One => TOP_BAR_HEIGHT_PX,
            HeaderRows::Two => TOP_BAR_HEIGHT_PX + frame::GAP_PX + NAVIGATION_ROW_PX,
        }
    }

    pub(super) fn chrome_height_px(&self) -> f32 {
        self.header_height_px() + frame::GAP_PX + ADDRESS_HEIGHT_PX
    }

    fn pinned_end(&self, cx: &mut Context<Self>) -> Option<Div> {
        let has_pinned = !self.current().pinned.is_empty();
        match (has_pinned, cx.has_active_drag()) {
            (true, _) => Some(separator(self.colors())),
            (false, true) => Some(self.drop_zone(
                divider_space(),
                Zones::whole(Spot::Top(Section::Pinned)),
                cx,
            )),
            (false, false) => None,
        }
    }

    fn row(&self, tab: &Tab, depth: usize, cx: &mut Context<Self>) -> AnyElement {
        let id = tab.id;
        let active = self.current().active == Some(id);
        let title = tab_title(tab);
        let colors = self.colors();
        let favicon = site_icon(&self.favicons, tab.url(), rgba(colors.muted), cx);
        let (row_active, row_hover) = (colors.row_active, colors.row_hover);
        let zones = Zones {
            upper: Spot::Before(Item::Tab(id)),
            lower: Spot::After(Item::Tab(id)),
        };
        let slot = self.tab_slot(div().id(("tab", id.0)), id, cx);
        let row = self
            .drop_zone(
                self.draggable(slot, Item::Tab(id), title.clone()),
                zones,
                cx,
            )
            .group(TAB_ROW_GROUP)
            .when(active, |row| row.bg(rgba(row_active)))
            .when(!active, |row| row.hover(|row| row.bg(rgba(row_hover))))
            .on_click(cx.listener(move |browser, _, _, cx| browser.activate(id, cx)))
            .on_mouse_down(
                MouseButton::Right,
                self.context_menu_opener(Target::Tab(id), cx),
            );
        let row = self.peekable(row, id, cx);
        let row = match self.density() {
            Density::Compact => compact_row(row).child(favicon),
            Density::Roomy => roomy_row(row, depth)
                .child(favicon)
                .child(div().flex_1().min_w_0().truncate().child(title))
                .children(self.sound_badge(id, cx))
                .child(
                    row_button(
                        ("close", id.0),
                        Icon::X.tinted(rgba(colors.muted)).size(px(CLOSE_ICON_PX)),
                        colors,
                    )
                    .when(!active, |button| {
                        button
                            .invisible()
                            .group_hover(TAB_ROW_GROUP, |button| button.visible())
                    })
                    .on_click(cx.listener(move |browser, _, _, cx| {
                        cx.stop_propagation();
                        browser.close(id, cx);
                    })),
                ),
        };
        row.into_any_element()
    }

    fn folder_row(&self, folder: &Folder, depth: usize, cx: &mut Context<Self>) -> AnyElement {
        let id = folder.id;
        let (chevron, icon) = match folder.collapsed {
            true => (Icon::ChevronRight, Icon::Folder),
            false => (Icon::ChevronDown, Icon::FolderOpen),
        };
        let colors = self.colors();
        let (row_hover, muted) = (colors.row_hover, rgba(colors.muted));
        let zones = Zones {
            upper: Spot::Before(Item::Folder(id)),
            lower: Spot::Into(id),
        };
        let row = self.draggable(
            div().id(("folder", id.0)),
            Item::Folder(id),
            folder.name.clone(),
        );
        let row = self
            .drop_zone(row, zones, cx)
            .hover(move |row| row.bg(rgba(row_hover)))
            .on_click(cx.listener(move |browser, _, _, cx| browser.toggle_folder(id, cx)));
        let row = match self.density() {
            Density::Compact => compact_row(row).child(icon.tinted(muted)),
            Density::Roomy => roomy_row(row, depth)
                .child(chevron.tinted(muted).size(px(CHEVRON_PX)))
                .child(icon.tinted(muted))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(folder.name.clone()),
                ),
        };
        row.into_any_element()
    }

    fn pinned_rows(
        &self,
        items: &[Pinned],
        depth: usize,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        items
            .iter()
            .flat_map(|item| match item {
                Pinned::Tab(tab) => vec![self.row(tab, depth, cx)],
                Pinned::Folder(folder) => {
                    let mut rows = vec![self.folder_row(folder, depth, cx)];
                    if !folder.collapsed {
                        rows.extend(self.pinned_rows(&folder.items, depth + 1, cx));
                    }
                    rows
                }
            })
            .collect()
    }

    fn new_tab_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors();
        let (row_hover, muted) = (colors.row_hover, rgba(colors.muted));
        let row = self.drop_zone(
            div().id("new-tab"),
            Zones::whole(Spot::Top(Section::Today)),
            cx,
        );
        let row = row
            .text_color(muted)
            .hover(move |row| row.bg(rgba(row_hover)))
            .on_click(cx.listener(|_, _, window, cx| window.dispatch_action(Box::new(NewTab), cx)));
        let plus = Icon::Plus.tinted(muted);
        match self.density() {
            Density::Compact => compact_row(row).child(plus),
            Density::Roomy => roomy_row(row, 0).child(plus).child(self.text(Text::NewTab)),
        }
    }

    fn space_buttons(&self, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let current = self.current().id;
        let colors = self.colors();
        let (row_active, row_hover) = (colors.row_active, colors.row_hover);
        let (text, muted) = (rgba(colors.text), rgba(colors.muted));
        self.store
            .session
            .spaces
            .iter()
            .map(|space| {
                let id = space.id;
                let active = id == current;
                let icon_color = match active {
                    true => text,
                    false => muted,
                };
                self.space_drop_zone(square_button(("space", id.0)), id, cx)
                    .when(active, |button| button.bg(rgba(row_active)).shadow_sm())
                    .when(!active, |button| {
                        button.hover(|button| button.bg(rgba(row_hover)))
                    })
                    .on_click(cx.listener(move |browser, _, _, cx| browser.switch_space(id, cx)))
                    .on_mouse_down(
                        MouseButton::Right,
                        self.context_menu_opener(Target::Space(id), cx),
                    )
                    .child(theme(space).icon.tinted(icon_color))
                    .into_any_element()
            })
            .collect()
    }

    fn settings_button(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let row_hover = colors.row_hover;
        square_button("sidebar-settings")
            .hover(move |button| button.bg(rgba(row_hover)))
            .on_click(cx.listener(|browser, _, window, cx| browser.toggle_settings(window, cx)))
            .child(Icon::Settings.tinted(rgba(colors.muted)))
    }

    fn profile_label(&self) -> Option<Div> {
        let profiles = self.store.profiles();
        let name = profiles.custom_name(&self.current().profile)?;
        Some(
            div()
                .flex_none()
                .max_w(px(PROFILE_LABEL_MAX_PX))
                .truncate()
                .font_weight(gpui::FontWeight::NORMAL)
                .opacity(PROFILE_LABEL_OPACITY)
                .child(name.to_owned()),
        )
    }

    fn space_name(&self) -> Div {
        frame::drag()
            .min_w_0()
            .flex()
            .items_center()
            .gap_1()
            .text_xs()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgba(self.colors().muted))
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .child(self.current().name.clone()),
            )
            .children(self.profile_label())
    }

    fn spaces_switcher(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let switcher = div().flex().gap_1().pt_2();
        let switcher = match self.density() {
            Density::Compact => switcher.flex_col().items_center(),
            Density::Roomy => switcher.child(self.space_name()),
        };
        switcher
            .children(self.space_buttons(cx))
            .children(self.downloads_button(cx))
            .child(self.settings_button(cx))
    }

    fn header_bar(&self, content: Vec<AnyElement>, window: &Window) -> AnyElement {
        let controls = self.layout().controls;
        let strip = frame::controls(controls, Flow::Row, self.colors(), window);
        frame::bar(controls.side, strip, content)
            .h(px(TOP_BAR_HEIGHT_PX))
            .into_any_element()
    }

    fn navigation_row(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .h(px(NAVIGATION_ROW_PX))
            .flex()
            .items_center()
            .gap(px(frame::GAP_PX))
            .child(self.navigation(cx))
            .child(frame::drag())
            .into_any_element()
    }

    fn header(&self, window: &Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let toggle = self.sidebar_button(cx).into_any_element();
        let drag = frame::drag().into_any_element();
        match self.header_rows() {
            HeaderRows::One => {
                let navigation = self.navigation(cx).into_any_element();
                vec![self.header_bar(vec![toggle, navigation, drag], window)]
            }
            HeaderRows::Two => vec![
                self.header_bar(vec![toggle, drag], window),
                self.navigation_row(cx),
            ],
        }
    }

    fn address_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap(px(frame::GAP_PX))
            .child(self.address_bar(cx).flex_1().min_w_0())
            .child(self.menu_button(cx))
    }

    fn compact_header(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let controls = frame::controls(self.layout().controls, Flow::Column, self.colors(), window);
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .child(controls)
            .child(self.sidebar_button(cx))
            .child(self.menu_button(cx))
    }

    fn chrome(&self, window: &Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        match self.density() {
            Density::Compact => vec![self.compact_header(window, cx).into_any_element()],
            Density::Roomy => {
                let mut chrome = self.header(window, cx);
                chrome.push(self.address_row(cx).into_any_element());
                chrome
            }
        }
    }

    fn player(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        match self.density() {
            Density::Compact => self.mini_player_button(cx).map(|button| {
                div()
                    .flex()
                    .justify_center()
                    .child(button)
                    .into_any_element()
            }),
            Density::Roomy => self.mini_player(cx),
        }
    }

    fn badge(&self) -> Option<AnyElement> {
        let badge = self.private_badge()?;
        let roomy = self.density() == Density::Roomy;
        roomy.then(|| div().flex().px_1().child(badge).into_any_element())
    }

    pub(super) fn sidebar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let space = self.current();
        let pinned = self.pinned_rows(&space.pinned, 0, cx);
        let today: Vec<AnyElement> = space.today.iter().map(|tab| self.row(tab, 0, cx)).collect();
        let chrome = !self.layout().top_bar.on();
        div()
            .id("sidebar")
            .on_hover(cx.listener(|browser, hovered: &bool, _, cx| {
                if !hovered {
                    browser.stop_peeking(cx);
                }
            }))
            .w(px(self.sidebar_width_px()))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .px(px(INSET_PX))
            .pb(px(INSET_PX))
            .when(!chrome, |sidebar| sidebar.pt(px(INSET_PX)))
            .gap(px(frame::GAP_PX))
            .when(chrome, |sidebar| sidebar.children(self.chrome(window, cx)))
            .children(self.badge())
            .child(self.favorites_grid(cx))
            .children(pinned)
            .children(self.pinned_end(cx))
            .child(
                div()
                    .id("tabs")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(self.new_tab_row(cx))
                    .children(today)
                    .child(frame::drag()),
            )
            .children(self.player(cx))
            .child(self.spaces_switcher(cx))
    }
}
