use std::path::PathBuf;

use gpui::{
    App, Context, Div, Entity, EventEmitter, FocusHandle, Focusable, KeyBinding, MouseButton,
    SharedString, Stateful, Subscription, Window, actions, div, prelude::*, px, rgba,
};
use url::Url;

use super::{
    downloads::{Control, DownloadId, Downloads, progress_bar},
    favicon::{Favicons, site_icon},
    field::{Changed, Field},
    icon::Icon,
    surface::{Colors, PANEL_CORNER_PX, PANEL_PADDING_PX, surface, tint},
    toolbar::icon_button,
};
use crate::{
    address::{Destination, destination},
    autocomplete::{completes, suggestions},
    config::Search,
    download::{Download, State},
    locale::{Locale, Text},
    storage::{
        search::{SearchResult, Source},
        session::TabId,
    },
};

actions!(command_bar, [SelectNext, SelectPrevious, Confirm, Dismiss]);

const CONTEXT: &str = "CommandBar";
const MAX_ITEMS: usize = 8;
const WIDTH_PX: f32 = 600.0;
const INPUT_TEXT_PX: f32 = 15.0;
const ROW_TEXT_PX: f32 = 13.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Target {
    NewTab,
    Current,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Everything(Target),
    Downloads,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Command {
    ToggleSidebar,
    Settings,
    SplitView,
    Unsplit,
    NewSpace,
    PinTab,
    UnpinTab,
    PinInNewFolder,
    ToggleFavorite,
    CloseTab,
    Reload,
    FindInPage,
    Print,
    SavePage,
    DevTools,
    Downloads,
    ReaderMode,
    DarkMode,
}

#[derive(Clone)]
pub enum Choice {
    Open(Url, Target),
    Activate(TabId),
    Restore(TabId),
    Run(Command),
    OpenFile(PathBuf),
    ShowFile(PathBuf),
}

pub enum Outcome {
    Chosen(Choice),
    Dismissed,
}

pub type Lookup = Box<dyn Fn(&str, &App) -> Vec<SearchResult>>;

pub struct Options {
    pub mode: Mode,
    pub text: String,
    pub colors: &'static Colors,
    pub search: Search,
    pub locale: Locale,
}

pub struct Sources {
    pub lookup: Lookup,
    pub favicons: Entity<Favicons>,
    pub downloads: Entity<Downloads>,
}

enum Mark {
    Icon(Icon),
    Site(String),
}

enum Detail {
    Hint(SharedString),
    Download(DownloadId, Download),
}

struct Item {
    mark: Mark,
    label: SharedString,
    detail: Detail,
    choice: Option<Choice>,
    completes: bool,
}

pub struct CommandBar {
    field: Entity<Field>,
    options: Options,
    sources: Sources,
    selected: usize,
    _subscriptions: [Subscription; 3],
}

const COMMANDS: [(Text, Command, Icon); 18] = [
    (Text::SplitView, Command::SplitView, Icon::Columns2),
    (Text::CloseSplit, Command::Unsplit, Icon::PanelLeftClose),
    (Text::NewSpace, Command::NewSpace, Icon::Layers),
    (Text::PinTab, Command::PinTab, Icon::Pin),
    (Text::UnpinTab, Command::UnpinTab, Icon::Pin),
    (
        Text::PinTabInNewFolder,
        Command::PinInNewFolder,
        Icon::Folder,
    ),
    (Text::ToggleFavorite, Command::ToggleFavorite, Icon::Star),
    (Text::CloseTab, Command::CloseTab, Icon::X),
    (Text::ToggleSidebar, Command::ToggleSidebar, Icon::PanelLeft),
    (Text::Settings, Command::Settings, Icon::Settings),
    (Text::ReloadPage, Command::Reload, Icon::RotateCw),
    (Text::FindInPage, Command::FindInPage, Icon::FileSearch),
    (Text::PrintPage, Command::Print, Icon::Printer),
    (Text::SavePage, Command::SavePage, Icon::FileDown),
    (Text::DeveloperTools, Command::DevTools, Icon::Code),
    (Text::Downloads, Command::Downloads, Icon::Download),
    (Text::ReaderMode, Command::ReaderMode, Icon::BookOpen),
    (Text::DarkModeOnThisSite, Command::DarkMode, Icon::Moon),
];

pub fn bindings() -> Vec<KeyBinding> {
    let context = Some(CONTEXT);
    vec![
        KeyBinding::new("down", SelectNext, context),
        KeyBinding::new("up", SelectPrevious, context),
        KeyBinding::new("enter", Confirm, context),
        KeyBinding::new("escape", Dismiss, context),
    ]
}

fn matches(haystack: &str, query: &str) -> bool {
    haystack.to_lowercase().contains(query)
}

fn placeholder(mode: Mode) -> Text {
    match mode {
        Mode::Everything(_) => Text::SearchOrEnterAddress,
        Mode::Downloads => Text::SearchDownloads,
    }
}

fn hint(text: &'static str) -> Detail {
    Detail::Hint(text.into())
}

fn place(target: Target) -> Text {
    match target {
        Target::NewTab => Text::NewTabTarget,
        Target::Current => Text::ThisTab,
    }
}

fn open_item(query: &str, typed: Destination, target: Target, locale: Locale) -> Item {
    let (mark, label, url) = match typed {
        Destination::Search(url) => (
            Mark::Icon(Icon::Search),
            locale.fill(Text::SearchFor, query),
            url,
        ),
        Destination::Address(url) => (
            Mark::Site(url.to_string()),
            locale.fill(Text::OpenAddress, url.as_str()),
            url,
        ),
    };
    Item {
        mark,
        label: label.into(),
        detail: hint(locale.text(place(target))),
        choice: Some(Choice::Open(url, target)),
        completes: false,
    }
}

fn site_item(url: Url, target: Target, locale: Locale) -> Item {
    Item {
        mark: Mark::Site(url.to_string()),
        label: url.host_str().unwrap_or_default().to_owned().into(),
        detail: hint(locale.text(Text::PopularSite)),
        choice: Some(Choice::Open(url, target)),
        completes: true,
    }
}

fn result_item(result: SearchResult, query: &str, target: Target, locale: Locale) -> Option<Item> {
    let (source, choice) = match (result.source, result.tab) {
        (Source::OpenTab, Some(id)) => (Text::SwitchToTab, Choice::Activate(id)),
        (Source::Pinned, Some(id)) => (Text::PinnedTab, Choice::Activate(id)),
        (Source::Archived, Some(id)) => (Text::ArchivedTab, Choice::Restore(id)),
        (Source::Favorite, _) => (
            Text::Favorite,
            Choice::Open(Url::parse(&result.url).ok()?, target),
        ),
        (_, _) => (
            Text::History,
            Choice::Open(Url::parse(&result.url).ok()?, target),
        ),
    };
    let mark = Mark::Site(result.url.clone());
    let completes = completes(query, &result.url);
    let label = match result.title.is_empty() {
        true => result.url,
        false => result.title,
    };
    Some(Item {
        mark,
        label: label.into(),
        detail: hint(locale.text(source)),
        choice: Some(choice),
        completes,
    })
}

fn command_items(query: &str, locale: Locale) -> impl Iterator<Item = Item> {
    COMMANDS
        .iter()
        .map(move |&(name, command, icon)| (locale.text(name), command, icon))
        .filter(move |(name, _, _)| matches(name, query))
        .map(move |(name, command, icon)| Item {
            mark: Mark::Icon(icon),
            label: name.into(),
            detail: hint(locale.text(Text::Command)),
            choice: Some(Choice::Run(command)),
            completes: false,
        })
}

fn download_item((id, download): (DownloadId, Download)) -> Item {
    let openable = download.state == State::Completed;
    Item {
        mark: Mark::Icon(Icon::Download),
        label: download.name().into(),
        choice: openable.then(|| Choice::OpenFile(download.path.clone())),
        detail: Detail::Download(id, download),
        completes: false,
    }
}

fn promote_completion(suggested: &mut Vec<Item>) -> Option<Item> {
    let index = suggested.iter().position(|item| item.completes)?;
    Some(suggested.remove(index))
}

impl EventEmitter<Outcome> for CommandBar {}

impl CommandBar {
    pub fn new(options: Options, sources: Sources, cx: &mut Context<Self>) -> Self {
        let placeholder = options.locale.text(placeholder(options.mode));
        let field = cx.new(|cx| Field::new(placeholder, &options.text, cx));
        let changes = cx.subscribe(&field, |bar: &mut Self, _, _: &Changed, cx| {
            bar.selected = 0;
            cx.notify();
        });
        let favicon_updates = cx.observe(&sources.favicons, |_, _, cx| cx.notify());
        let download_updates = cx.observe(&sources.downloads, |_, _, cx| cx.notify());
        Self {
            field,
            options,
            sources,
            selected: 0,
            _subscriptions: [changes, favicon_updates, download_updates],
        }
    }

    pub fn mode(&self) -> Mode {
        self.options.mode
    }

    fn downloads(&self, text: &str, cx: &App) -> Vec<Item> {
        let shelf = self.sources.downloads.read(cx).shelf(text);
        shelf.into_iter().map(download_item).collect()
    }

    fn everything(&self, text: &str, target: Target, cx: &App) -> Vec<Item> {
        let query = text.to_lowercase();
        let searching = !text.is_empty();
        let locale = self.options.locale;
        let typed = searching.then(|| destination(text, &self.options.search));
        let address = match &typed {
            Some(Destination::Address(url)) => Some(url.as_str()),
            _ => None,
        };
        let results = (self.sources.lookup)(text, cx);
        let known = results.iter().map(|result| result.url.as_str());
        let sites = suggestions(text, known.chain(address));
        let addressed = address.is_some();
        let mut suggested: Vec<Item> = results
            .into_iter()
            .filter_map(|result| result_item(result, text, target, locale))
            .chain(sites.into_iter().map(|url| site_item(url, target, locale)))
            .collect();
        let top = match addressed {
            true => None,
            false => promote_completion(&mut suggested),
        };
        let open = typed.map(|typed| open_item(text, typed, target, locale));
        let downloads = searching.then(|| self.downloads(text, cx));
        top.into_iter()
            .chain(open)
            .chain(suggested)
            .chain(downloads.into_iter().flatten())
            .chain(command_items(&query, locale))
            .collect()
    }

    fn items(&self, cx: &App) -> Vec<Item> {
        let text = self.field.read(cx).text().trim().to_owned();
        let mut items = match self.options.mode {
            Mode::Everything(target) => self.everything(&text, target, cx),
            Mode::Downloads => self.downloads(&text, cx),
        };
        items.truncate(MAX_ITEMS);
        items
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        let count = self.items(cx).len();
        self.selected = (self.selected + 1).min(count.saturating_sub(1));
        cx.notify();
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = self.selected.saturating_sub(1);
        cx.notify();
    }

    fn confirm(&mut self, _: &Confirm, _: &mut Window, cx: &mut Context<Self>) {
        let chosen = self.items(cx).into_iter().nth(self.selected);
        if let Some(choice) = chosen.and_then(|item| item.choice) {
            cx.emit(Outcome::Chosen(choice));
        }
    }

    fn dismiss(&mut self, _: &Dismiss, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(Outcome::Dismissed);
    }

    fn clear_downloads(&mut self, cx: &mut Context<Self>) {
        self.sources.downloads.update(cx, Downloads::clear);
        self.selected = 0;
    }

    fn download_summary(&self, label: SharedString, download: &Download) -> Div {
        let colors = self.options.colors;
        let progress = download.progress().filter(|_| download.active());
        div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_1()
            .child(div().truncate().child(label))
            .children(progress.map(|fraction| progress_bar(fraction, colors)))
            .child(
                div()
                    .text_xs()
                    .text_color(rgba(colors.muted))
                    .child(download.status()),
            )
    }

    fn control_button(
        &self,
        index: usize,
        id: DownloadId,
        control: Control,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let (name, icon) = match control {
            Control::Pause => ("download-pause", Icon::Pause),
            Control::Resume => ("download-resume", Icon::Play),
            Control::Cancel => ("download-cancel", Icon::X),
        };
        icon_button((name, index), icon, self.options.colors).on_click(cx.listener(
            move |bar, _, _, cx| {
                cx.stop_propagation();
                bar.sources.downloads.read(cx).control(id, control);
            },
        ))
    }

    fn folder_button(
        &self,
        index: usize,
        download: &Download,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let path = download.path.clone();
        icon_button(
            ("download-folder", index),
            Icon::FolderOpen,
            self.options.colors,
        )
        .on_click(cx.listener(move |_, _, _, cx| {
            cx.stop_propagation();
            cx.emit(Outcome::Chosen(Choice::ShowFile(path.clone())));
        }))
    }

    fn download_actions(
        &self,
        index: usize,
        id: DownloadId,
        download: &Download,
        cx: &mut Context<Self>,
    ) -> Vec<Stateful<Div>> {
        let controls = match (download.state, download.pausable) {
            (State::InProgress, true) => vec![Control::Pause, Control::Cancel],
            (State::InProgress, false) => vec![Control::Cancel],
            (State::Paused, _) => vec![Control::Resume, Control::Cancel],
            (State::Completed, _) => return vec![self.folder_button(index, download, cx)],
            (State::Interrupted | State::Cancelled, _) => Vec::new(),
        };
        controls
            .into_iter()
            .map(|control| self.control_button(index, id, control, cx))
            .collect()
    }

    fn row(&self, index: usize, item: Item, cx: &mut Context<Self>) -> Stateful<Div> {
        let selected = index == self.selected;
        let muted = rgba(self.options.colors.muted);
        let ink = self.options.colors.text;
        let mark = match &item.mark {
            Mark::Icon(icon) => icon.tinted(muted).into_any_element(),
            Mark::Site(url) => site_icon(&self.sources.favicons, url, muted, cx),
        };
        let row = div()
            .id(("command-item", index))
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .px_2()
            .py_1()
            .text_size(px(ROW_TEXT_PX))
            .rounded_md()
            .when(selected, |row| row.bg(tint(ink, 0x2e)))
            .hover(|row| row.bg(tint(ink, 0x1a)))
            .when_some(item.choice, |row, choice| {
                row.on_click(
                    cx.listener(move |_, _, _, cx| cx.emit(Outcome::Chosen(choice.clone()))),
                )
            })
            .child(mark);
        match item.detail {
            Detail::Hint(hint) => row
                .child(div().flex_1().truncate().child(item.label))
                .child(div().text_xs().text_color(muted).child(hint)),
            Detail::Download(id, download) => row
                .child(self.download_summary(item.label, &download))
                .children(self.download_actions(index, id, &download, cx)),
        }
    }

    fn downloads_header(&self, empty: bool, cx: &mut Context<Self>) -> Div {
        let colors = self.options.colors;
        let muted = rgba(colors.muted);
        let downloads = self.sources.downloads.read(cx);
        let locale = self.options.locale;
        let note = match (empty, downloads.is_empty()) {
            (false, _) => Text::Downloads,
            (true, true) => Text::NoDownloadsYet,
            (true, false) => Text::NoMatchingDownloads,
        };
        let clearable = downloads.finished();
        div()
            .flex()
            .items_center()
            .justify_between()
            .px_2()
            .pt_1()
            .text_xs()
            .text_color(muted)
            .child(locale.text(note))
            .when(clearable, |header| {
                header.child(
                    div()
                        .id("downloads-clear")
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .hover(|button| button.bg(rgba(colors.overlay)))
                        .cursor_pointer()
                        .on_click(cx.listener(|bar, _, _, cx| bar.clear_downloads(cx)))
                        .child(locale.text(Text::ClearList)),
                )
            })
    }
}

impl Focusable for CommandBar {
    fn focus_handle(&self, cx: &gpui::App) -> FocusHandle {
        self.field.focus_handle(cx)
    }
}

impl Render for CommandBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows: Vec<_> = self
            .items(cx)
            .into_iter()
            .enumerate()
            .map(|(index, item)| self.row(index, item, cx))
            .collect();
        let header = (self.options.mode == Mode::Downloads)
            .then(|| self.downloads_header(rows.is_empty(), cx));
        let colors = self.options.colors;
        surface(colors)
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .w(px(WIDTH_PX))
            .flex()
            .flex_col()
            .gap_0p5()
            .p(px(PANEL_PADDING_PX))
            .rounded(px(PANEL_CORNER_PX))
            .child(
                div()
                    .px_2()
                    .py_1p5()
                    .mb_0p5()
                    .text_size(px(INPUT_TEXT_PX))
                    .border_b_1()
                    .border_color(rgba(colors.line))
                    .child(self.field.clone()),
            )
            .children(header)
            .children(rows)
    }
}
