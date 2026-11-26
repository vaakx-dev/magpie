use gpui::{AnyElement, Context, Div, SharedString, Stateful, Window, div, prelude::*, px, rgba};
use url::Url;

use super::{
    Find, NewPrivateWindow, NewTab, NewWindow, OpenDevTools, Print, SavePage, ToggleFullscreen,
    ToggleReader,
    browser::{Browser, Popover},
    command::Target,
    favicon::site_icon,
    icon::Icon,
    sidebar::tab_title,
    surface::{Colors, PANEL_PADDING_PX, ROW_HEIGHT_PX, panel, tint},
    toolbar::{BUTTON_PX, icon_button},
};
use crate::{
    config::Switch,
    darkmode::Mode,
    internal::{Page, settings::Section},
    locale::Text,
    storage::{
        history::Visit,
        session::{Folder, FolderId, Link, Pinned, Tab},
    },
    zoom::{Step, Zoom},
};
use magpie_engine::blocker::{site, web_site};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    History,
    ClosedTabs,
    Bookmarks,
    Folder(FolderId),
}

type Run = fn(&mut Browser, &mut Window, &mut Context<Browser>);

enum Action {
    Run { run: Run, shortcut: &'static str },
    Enter(Layer),
}

struct Item {
    id: &'static str,
    icon: Icon,
    label: Text,
    action: Action,
}

const ZOOM_LEVEL_WIDTH_PX: f32 = 48.0;
const DIVIDER_INSET_PX: f32 = 6.0;
const DIVIDER_SIDE_PX: f32 = DIVIDER_INSET_PX - PANEL_PADDING_PX;
const LIST_MAX_HEIGHT_PX: f32 = 420.0;
const RECENT_HISTORY_COUNT: usize = 15;
const REOPEN_CLOSED_TAB_SHORTCUT: &str = "Ctrl+Shift+T";

const NEW_TAB: Item = Item {
    id: "menu-new-tab",
    icon: Icon::SquarePlus,
    label: Text::NewTab,
    action: Action::Run {
        run: |browser, window, cx| browser.new_tab(&NewTab, window, cx),
        shortcut: "Ctrl+T",
    },
};

const NEW_WINDOW: Item = Item {
    id: "menu-new-window",
    icon: Icon::AppWindow,
    label: Text::NewWindow,
    action: Action::Run {
        run: |browser, window, cx| browser.new_window(&NewWindow, window, cx),
        shortcut: "Ctrl+N",
    },
};

const NEW_PRIVATE_WINDOW: Item = Item {
    id: "menu-new-private-window",
    icon: Icon::EyeOff,
    label: Text::NewPrivateWindow,
    action: Action::Run {
        run: |browser, window, cx| browser.new_private_window(&NewPrivateWindow, window, cx),
        shortcut: "Ctrl+Shift+N",
    },
};

const HISTORY: Item = Item {
    id: "menu-history",
    icon: Icon::History,
    label: Text::History,
    action: Action::Enter(Layer::History),
};

const BOOKMARKS: Item = Item {
    id: "menu-bookmarks",
    icon: Icon::Bookmark,
    label: Text::Bookmarks,
    action: Action::Enter(Layer::Bookmarks),
};

const RECENTLY_CLOSED_TABS: Item = Item {
    id: "menu-recently-closed-tabs",
    icon: Icon::RotateCw,
    label: Text::RecentlyClosedTabs,
    action: Action::Enter(Layer::ClosedTabs),
};

const CLEAR_RECENT_HISTORY: Item = Item {
    id: "menu-clear-recent-history",
    icon: Icon::Trash,
    label: Text::ClearRecentHistory,
    action: Action::Run {
        run: |browser, _, cx| browser.show_settings(Section::Privacy, cx),
        shortcut: "",
    },
};

const MANAGE_HISTORY: Item = Item {
    id: "menu-manage-history",
    icon: Icon::History,
    label: Text::ManageHistory,
    action: Action::Run {
        run: |browser, _, cx| browser.show_page(Page::History, cx),
        shortcut: "Ctrl+H",
    },
};

const REOPEN_ALL_TABS: Item = Item {
    id: "menu-reopen-all-tabs",
    icon: Icon::Layers,
    label: Text::ReopenAllTabs,
    action: Action::Run {
        run: |browser, _, cx| browser.reopen_all_closed(cx),
        shortcut: "",
    },
};

const MANAGE_BOOKMARKS: Item = Item {
    id: "menu-manage-bookmarks",
    icon: Icon::Bookmark,
    label: Text::ManageBookmarks,
    action: Action::Run {
        run: |browser, _, cx| browser.show_page(Page::Bookmarks, cx),
        shortcut: "",
    },
};

const SPLIT_VIEW: Item = Item {
    id: "menu-split-view",
    icon: Icon::Columns2,
    label: Text::SplitViewMenu,
    action: Action::Run {
        run: |browser, window, cx| browser.toggle_split(window, cx),
        shortcut: "Ctrl+Shift+\\",
    },
};

const FIND_IN_PAGE: Item = Item {
    id: "menu-find-in-page",
    icon: Icon::FileSearch,
    label: Text::FindInPageMenu,
    action: Action::Run {
        run: |browser, window, cx| browser.open_find(&Find, window, cx),
        shortcut: "Ctrl+F",
    },
};

const PRINT: Item = Item {
    id: "menu-print",
    icon: Icon::Printer,
    label: Text::Print,
    action: Action::Run {
        run: |browser, window, cx| browser.print(&Print, window, cx),
        shortcut: "Ctrl+P",
    },
};

const SAVE_PAGE: Item = Item {
    id: "menu-save-page",
    icon: Icon::FileDown,
    label: Text::SavePageAsMenu,
    action: Action::Run {
        run: |browser, window, cx| browser.save_page(&SavePage, window, cx),
        shortcut: "Ctrl+S",
    },
};

const DEVELOPER_TOOLS: Item = Item {
    id: "menu-developer-tools",
    icon: Icon::Code,
    label: Text::DeveloperToolsMenu,
    action: Action::Run {
        run: |browser, window, cx| browser.open_dev_tools(&OpenDevTools, window, cx),
        shortcut: "F12",
    },
};

const DOWNLOADS: Item = Item {
    id: "menu-downloads",
    icon: Icon::Download,
    label: Text::Downloads,
    action: Action::Run {
        run: |browser, _, cx| browser.show_page(Page::Downloads, cx),
        shortcut: "Ctrl+J",
    },
};

const PASSWORDS: Item = Item {
    id: "menu-passwords",
    icon: Icon::KeyRound,
    label: Text::Passwords,
    action: Action::Run {
        run: |browser, window, cx| browser.show_passwords(window, cx),
        shortcut: "",
    },
};

const SETTINGS: Item = Item {
    id: "menu-settings",
    icon: Icon::Settings,
    label: Text::Settings,
    action: Action::Run {
        run: |browser, _, cx| browser.show_page(Page::Settings, cx),
        shortcut: "Ctrl+,",
    },
};

const AD_BLOCKER: Item = Item {
    id: "menu-ad-blocker",
    icon: Icon::Shield,
    label: Text::AdBlocker,
    action: Action::Run {
        run: |browser, _, cx| browser.show_settings(Section::Privacy, cx),
        shortcut: "",
    },
};

const READER_MODE: Item = Item {
    id: "menu-reader-mode",
    icon: Icon::BookOpen,
    label: Text::ReaderMode,
    action: Action::Run {
        run: |browser, window, cx| browser.toggle_reader(&ToggleReader, window, cx),
        shortcut: "Alt+R",
    },
};

const FORCE_DARK_MODE: Item = Item {
    id: "menu-force-dark-mode",
    icon: Icon::Moon,
    label: Text::ForceDarkMode,
    action: Action::Run {
        run: |browser, _, cx| browser.show_settings(Section::AddOns, cx),
        shortcut: "",
    },
};

const CLEAR_BROWSING_DATA: Item = Item {
    id: "menu-clear-browsing-data",
    icon: Icon::Trash,
    label: Text::ClearBrowsingDataMenu,
    action: Action::Run {
        run: |browser, _, cx| browser.show_settings(Section::Privacy, cx),
        shortcut: "",
    },
};

fn rule(colors: &Colors) -> Div {
    div().h(px(1.0)).bg(rgba(colors.line))
}

pub(super) fn divider(colors: &Colors) -> Div {
    rule(colors).my_0p5().mx(px(DIVIDER_SIDE_PX))
}

fn underline(colors: &Colors) -> Div {
    rule(colors)
        .absolute()
        .top(px(ROW_HEIGHT_PX))
        .left(px(DIVIDER_SIDE_PX))
        .right(px(DIVIDER_SIDE_PX))
}

pub(super) fn heading(label: &'static str, colors: &Colors) -> Div {
    div()
        .px_2()
        .pt_1()
        .pb_0p5()
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgba(colors.muted))
        .child(label)
}

fn shortcut(keys: &'static str, colors: &Colors) -> Div {
    div().text_xs().text_color(rgba(colors.muted)).child(keys)
}

fn chevron(colors: &Colors) -> impl IntoElement {
    Icon::ChevronRight.tinted(rgba(colors.muted))
}

fn zoom_level(zoom: Zoom, colors: &Colors) -> Stateful<Div> {
    let hover = colors.overlay;
    div()
        .id("menu-zoom-reset")
        .h(px(BUTTON_PX))
        .min_w(px(ZOOM_LEVEL_WIDTH_PX))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .hover(move |level| level.bg(rgba(hover)))
        .child(zoom.to_string())
}

fn entry(leading: impl IntoElement, label: impl Into<SharedString>) -> Div {
    div()
        .h(px(ROW_HEIGHT_PX))
        .px_2()
        .flex()
        .items_center()
        .gap_2()
        .rounded_md()
        .child(leading)
        .child(div().flex_1().min_w_0().truncate().child(label.into()))
}

pub(super) fn line(icon: Icon, label: impl Into<SharedString>, colors: &Colors) -> Div {
    entry(icon.tinted(rgba(colors.muted)), label)
}

fn pressable(row: Stateful<Div>, colors: &Colors) -> Stateful<Div> {
    let hover = colors.overlay;
    row.cursor_pointer().hover(move |row| row.bg(rgba(hover)))
}

fn list(id: &'static str, rows: impl IntoIterator<Item = impl IntoElement>) -> Stateful<Div> {
    div()
        .id(id)
        .max_h(px(LIST_MAX_HEIGHT_PX))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap_0p5()
        .children(rows)
}

fn section(label: &'static str, rows: Vec<Stateful<Div>>, colors: &Colors) -> Option<Div> {
    (!rows.is_empty()).then(|| {
        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .child(heading(label, colors))
            .children(rows)
    })
}

impl Browser {
    pub(super) fn show_menu(&mut self, cx: &mut Context<Self>) {
        self.popover = Some(Popover::Menu(Vec::new()));
        cx.notify();
    }

    pub(super) fn menu_button(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        icon_button("menu", Icon::Menu, self.colors())
            .on_click(cx.listener(|browser, _, _, cx| browser.show_menu(cx)))
    }

    fn enter(&mut self, layer: Layer, cx: &mut Context<Self>) {
        let Some(Popover::Menu(layers)) = &mut self.popover else {
            return;
        };
        layers.push(layer);
        cx.notify();
    }

    fn leave(&mut self, cx: &mut Context<Self>) {
        let Some(Popover::Menu(layers)) = &mut self.popover else {
            return;
        };
        layers.pop();
        cx.notify();
    }

    fn pick(
        &self,
        row: Stateful<Div>,
        run: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        pressable(row, self.colors()).on_click(cx.listener(move |browser, _, window, cx| {
            browser.dismiss_popover(window, cx);
            run(browser, window, cx);
        }))
    }

    fn drill(&self, row: Stateful<Div>, layer: Layer, cx: &mut Context<Self>) -> Stateful<Div> {
        pressable(row, self.colors())
            .on_click(cx.listener(move |browser, _, _, cx| browser.enter(layer, cx)))
            .child(chevron(self.colors()))
    }

    fn item(&self, item: Item, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let row = line(item.icon, self.text(item.label), colors).id(item.id);
        match item.action {
            Action::Run {
                run,
                shortcut: keys,
            } => self
                .pick(row, run, cx)
                .when(!keys.is_empty(), |row| row.child(shortcut(keys, colors))),
            Action::Enter(layer) => self.drill(row, layer, cx),
        }
    }

    fn favicon(&self, url: &str, cx: &mut Context<Self>) -> AnyElement {
        site_icon(&self.favicons, url, rgba(self.colors().muted), cx)
    }

    fn layer_header(&self, title: impl Into<SharedString>, cx: &mut Context<Self>) -> Div {
        let back = icon_button("menu-back", Icon::ChevronLeft, self.colors())
            .cursor_pointer()
            .on_click(cx.listener(|browser, _, _, cx| browser.leave(cx)));
        let title = div()
            .flex_1()
            .min_w_0()
            .truncate()
            .text_center()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .child(title.into());
        div()
            .relative()
            .h(px(ROW_HEIGHT_PX))
            .flex()
            .items_center()
            .child(back)
            .child(title)
            .child(div().flex_none().w(px(BUTTON_PX)))
            .child(underline(self.colors()))
    }

    fn subview(&self, title: impl Into<SharedString>, cx: &mut Context<Self>) -> Div {
        panel(self.colors()).child(self.layer_header(title, cx))
    }

    fn blocker_item(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let shield = self.runtime.shield();
        let page = self.active_page(cx).filter(|url| site(url).is_some());
        let item = self.item(AD_BLOCKER, cx);
        match (shield.enabled(), page) {
            (false, _) => item.child(shortcut(self.text(Text::Off), self.colors())),
            (true, None) => item,
            (true, Some(page)) => {
                let blocking = Switch::from(!shield.paused_on(&page));
                item.child(self.toggle(
                    "menu-ad-blocker-site",
                    blocking,
                    cx,
                    move |browser, _, cx| browser.toggle_shield(&page, cx),
                ))
            }
        }
    }

    fn dark_mode_item(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let item = self.item(FORCE_DARK_MODE, cx);
        let Some(page) = self.active_page(cx).filter(|url| web_site(url).is_some()) else {
            return item;
        };
        let dark = Switch::from(self.settings().dark_mode.mode(&page) != Mode::Off);
        item.child(
            self.toggle("menu-dark-mode-site", dark, cx, |browser, _, cx| {
                browser.toggle_dark_mode(cx)
            }),
        )
    }

    fn split_item(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let ink = self.colors().text;
        self.item(SPLIT_VIEW, cx)
            .when(self.split_active(), |item| item.bg(tint(ink, 0x24)))
    }

    fn stepping(&self, button: Stateful<Div>, step: Step, cx: &mut Context<Self>) -> Stateful<Div> {
        button
            .cursor_pointer()
            .on_click(cx.listener(move |browser, _, _, cx| browser.step_zoom(step, cx)))
    }

    fn fullscreen_button(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        icon_button("menu-fullscreen", Icon::Maximize, self.colors())
            .cursor_pointer()
            .on_click(cx.listener(|browser, _, window, cx| {
                browser.dismiss_popover(window, cx);
                browser.toggle_fullscreen(&ToggleFullscreen, window, cx);
            }))
    }

    fn zoom_item(&self, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        let zoom = self
            .active_view()
            .map_or(self.settings().default_zoom, |view| view.read(cx).zoom());
        let smaller = icon_button("menu-zoom-out", Icon::Minus, colors);
        let reset = zoom_level(zoom, colors);
        let larger = icon_button("menu-zoom-in", Icon::Plus, colors);
        line(Icon::ZoomIn, self.text(Text::Zoom), colors)
            .child(self.stepping(smaller, Step::Out, cx))
            .child(self.stepping(reset, Step::Reset, cx))
            .child(self.stepping(larger, Step::In, cx))
            .child(self.fullscreen_button(cx))
    }

    fn root_view(&self, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        panel(colors)
            .child(self.item(NEW_TAB, cx))
            .child(self.item(NEW_WINDOW, cx))
            .child(self.item(NEW_PRIVATE_WINDOW, cx))
            .child(divider(colors))
            .child(self.item(HISTORY, cx))
            .child(self.item(BOOKMARKS, cx))
            .child(self.item(DOWNLOADS, cx))
            .child(self.item(PASSWORDS, cx))
            .child(self.split_item(cx))
            .child(divider(colors))
            .child(self.zoom_item(cx))
            .child(divider(colors))
            .child(self.item(FIND_IN_PAGE, cx))
            .child(self.item(READER_MODE, cx))
            .child(self.item(PRINT, cx))
            .child(self.item(SAVE_PAGE, cx))
            .child(self.item(DEVELOPER_TOOLS, cx))
            .child(divider(colors))
            .child(self.blocker_item(cx))
            .child(self.dark_mode_item(cx))
            .child(self.item(CLEAR_BROWSING_DATA, cx))
            .child(divider(colors))
            .child(self.item(SETTINGS, cx))
    }

    fn closed_tabs_item(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        match self.closed.is_empty() {
            true => line(
                RECENTLY_CLOSED_TABS.icon,
                self.text(Text::RecentlyClosedTabs),
                colors,
            )
            .id(RECENTLY_CLOSED_TABS.id)
            .text_color(rgba(colors.faint))
            .child(Icon::ChevronRight.tinted(rgba(colors.faint))),
            false => self.item(RECENTLY_CLOSED_TABS, cx),
        }
    }

    fn visit_row(
        &self,
        index: usize,
        visit: &Visit,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        let url = Url::parse(&visit.url).ok()?;
        let title = match visit.title.is_empty() {
            true => visit.url.clone(),
            false => visit.title.clone(),
        };
        let row = entry(self.favicon(&visit.url, cx), title).id(("menu-visit", index));
        Some(self.pick(
            row,
            move |browser, _, cx| browser.open(url.clone(), Target::Current, cx),
            cx,
        ))
    }

    fn history_view(&self, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        let visits: Vec<Stateful<Div>> = self
            .store
            .history()
            .recent(RECENT_HISTORY_COUNT)
            .into_iter()
            .enumerate()
            .filter_map(|(index, visit)| self.visit_row(index, visit, cx))
            .collect();
        self.subview(self.text(Text::History), cx)
            .child(self.closed_tabs_item(cx))
            .child(self.item(CLEAR_RECENT_HISTORY, cx))
            .when(!visits.is_empty(), |view| {
                view.child(divider(colors))
                    .child(heading(self.text(Text::RecentHistory), colors))
                    .child(list("menu-recent-history", visits))
            })
            .child(divider(colors))
            .child(self.item(MANAGE_HISTORY, cx))
    }

    fn closed_tab_row(&self, index: usize, tab: &Tab, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let row = entry(self.favicon(tab.url(), cx), tab_title(tab)).id(("menu-closed-tab", index));
        self.pick(
            row,
            move |browser, _, cx| browser.reopen_closed(index, cx),
            cx,
        )
        .when(index == 0, |row| {
            row.child(shortcut(REOPEN_CLOSED_TAB_SHORTCUT, colors))
        })
    }

    fn closed_tabs_view(&self, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        let rows: Vec<Stateful<Div>> = self
            .closed
            .newest_first()
            .enumerate()
            .map(|(index, closed)| self.closed_tab_row(index, &closed.tab, cx))
            .collect();
        self.subview(self.text(Text::RecentlyClosedTabs), cx)
            .child(list("menu-closed-tabs", rows))
            .child(divider(colors))
            .child(self.item(REOPEN_ALL_TABS, cx))
    }

    fn favorite_row(&self, index: usize, link: &Link, cx: &mut Context<Self>) -> Stateful<Div> {
        let row =
            entry(self.favicon(&link.url, cx), link.title.clone()).id(("menu-favorite", index));
        self.pick(
            row,
            move |browser, _, cx| browser.open_favorite(index, cx),
            cx,
        )
    }

    fn pinned_tab_row(&self, tab: &Tab, cx: &mut Context<Self>) -> Stateful<Div> {
        let id = tab.id;
        let row = entry(self.favicon(tab.url(), cx), tab_title(tab)).id(("menu-pinned-tab", id.0));
        self.pick(row, move |browser, _, cx| browser.activate(id, cx), cx)
    }

    fn folder_entry(&self, folder: &Folder, cx: &mut Context<Self>) -> Stateful<Div> {
        let row =
            line(Icon::Folder, folder.name.clone(), self.colors()).id(("menu-folder", folder.id.0));
        self.drill(row, Layer::Folder(folder.id), cx)
    }

    fn pinned_entries(&self, items: &[Pinned], cx: &mut Context<Self>) -> Vec<Stateful<Div>> {
        items
            .iter()
            .map(|item| match item {
                Pinned::Tab(tab) => self.pinned_tab_row(tab, cx),
                Pinned::Folder(folder) => self.folder_entry(folder, cx),
            })
            .collect()
    }

    fn bookmarks_view(&self, cx: &mut Context<Self>) -> Div {
        let colors = self.colors();
        let space = self.current();
        let favorites: Vec<Stateful<Div>> = space
            .favorites
            .iter()
            .enumerate()
            .map(|(index, link)| self.favorite_row(index, link, cx))
            .collect();
        let pinned = self.pinned_entries(&space.pinned, cx);
        let sections = [
            section(self.text(Text::Favorites), favorites, colors),
            section(self.text(Text::Pinned), pinned, colors),
        ];
        let separated = sections
            .into_iter()
            .flatten()
            .enumerate()
            .flat_map(|(index, section)| [(index > 0).then(|| divider(colors)), Some(section)])
            .flatten();
        self.subview(self.text(Text::Bookmarks), cx)
            .child(list("menu-bookmarks-list", separated))
            .child(divider(colors))
            .child(self.item(MANAGE_BOOKMARKS, cx))
    }

    fn folder_view(&self, id: FolderId, cx: &mut Context<Self>) -> Div {
        match self.current().folder(id) {
            Some(folder) => self.subview(folder.name.clone(), cx).child(list(
                "menu-folder-list",
                self.pinned_entries(&folder.items, cx),
            )),
            None => self.bookmarks_view(cx),
        }
    }

    pub(super) fn menu(&self, layer: Option<Layer>, cx: &mut Context<Self>) -> Div {
        match layer {
            None => self.root_view(cx),
            Some(Layer::History) => self.history_view(cx),
            Some(Layer::ClosedTabs) => self.closed_tabs_view(cx),
            Some(Layer::Bookmarks) => self.bookmarks_view(cx),
            Some(Layer::Folder(id)) => self.folder_view(id, cx),
        }
    }
}
