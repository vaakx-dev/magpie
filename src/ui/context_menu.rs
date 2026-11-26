use gpui::{
    App, ClipboardItem, Context, Div, MouseDownEvent, Pixels, Point, SharedString, Stateful,
    Window, anchored, div, prelude::*, px, rgba,
};

use super::{
    browser::{Browser, Popover},
    icon::Icon,
    menu::{divider, heading, line},
    session::Partner,
    space::theme,
    surface::{Colors, PANEL_CORNER_PX, panel},
};
use crate::{
    internal::settings::Section as SettingsSection,
    locale::Text,
    storage::{
        arrange::Item,
        profile::ProfileId,
        session::{self, Section, Space, SpaceId, TabId},
    },
};

const WIDTH_PX: f32 = 232.0;
const WINDOW_MARGIN_PX: f32 = 8.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Tab(TabId),
    Favorite(usize),
    Space(SpaceId),
}

pub struct Menu {
    target: Target,
    position: Point<Pixels>,
}

#[derive(Clone)]
enum Action {
    Move(TabId, Section),
    ToggleFavorite(TabId),
    EditFavorite(usize),
    RemoveFavorite(usize),
    Duplicate(TabId),
    SplitActive,
    Split(Partner),
    ToggleSound(TabId),
    MoveToSpace(TabId, SpaceId),
    AssignProfile(SpaceId, ProfileId),
    ManageProfiles,
    Copy(String),
    Close(TabId),
}

struct Entry {
    icon: Icon,
    label: SharedString,
    action: Action,
}

struct Group {
    heading: Option<&'static str>,
    entries: Vec<Entry>,
}

fn entry(icon: Icon, label: impl Into<SharedString>, action: Action) -> Entry {
    Entry {
        icon,
        label: label.into(),
        action,
    }
}

fn group(entries: Vec<Entry>) -> Group {
    Group {
        heading: None,
        entries,
    }
}

fn pin_entry(id: TabId, section: Section) -> Entry {
    match section {
        Section::Today => entry(Icon::Pin, "Pin tab", Action::Move(id, Section::Pinned)),
        Section::Pinned => entry(Icon::Pin, "Unpin tab", Action::Move(id, Section::Today)),
    }
}

fn favorite_entry(id: TabId, favorited: bool) -> Entry {
    let label = match favorited {
        true => "Remove from Favorites",
        false => "Add to Favorites",
    };
    entry(Icon::Star, label, Action::ToggleFavorite(id))
}

fn sound_entry(id: TabId, muted: bool) -> Entry {
    let (icon, label) = match muted {
        true => (Icon::Volume, "Unmute tab"),
        false => (Icon::VolumeMuted, "Mute tab"),
    };
    entry(icon, label, Action::ToggleSound(id))
}

fn split_entry(action: Action) -> Entry {
    entry(Icon::Columns2, "Open in Split View", action)
}

fn copy_entry(url: &str) -> Entry {
    entry(Icon::Link, "Copy URL", Action::Copy(url.to_owned()))
}

fn card(colors: &Colors) -> Div {
    panel(colors)
        .w(px(WIDTH_PX))
        .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
}

impl Browser {
    pub(super) fn context_menu_opener(
        &self,
        target: Target,
        cx: &mut Context<Self>,
    ) -> impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static {
        cx.listener(move |browser, event: &MouseDownEvent, _, cx| {
            browser.popover = Some(Popover::Context(Menu {
                target,
                position: event.position,
            }));
            cx.notify();
        })
    }

    fn split_action(&self, id: TabId) -> Option<Action> {
        match self.current().active == Some(id) {
            true => (!self.split_active()).then_some(Action::SplitActive),
            false => self.partner(id).map(Action::Split),
        }
    }

    fn space_group(&self, id: TabId) -> Group {
        let current = self.current().id;
        let entries = self
            .store
            .session
            .spaces
            .iter()
            .filter(|space| space.id != current)
            .map(|space| {
                let action = Action::MoveToSpace(id, space.id);
                entry(theme(space).icon, space.name.clone(), action)
            })
            .collect();
        Group {
            heading: Some("Move to Space"),
            entries,
        }
    }

    fn tab_groups(&self, id: TabId, cx: &App) -> Vec<Group> {
        let space = self.current();
        let (Some(tab), Some(section)) = (space.tab(id), space.section(id)) else {
            return Vec::new();
        };
        let favorited = space.favorites.iter().any(|link| link.url == tab.url());
        let arrange = [
            Some(pin_entry(id, section)),
            Some(favorite_entry(id, favorited)),
            Some(entry(Icon::Copy, "Duplicate tab", Action::Duplicate(id))),
            self.split_action(id).map(split_entry),
            self.muted(id, cx).map(|muted| sound_entry(id, muted)),
        ];
        vec![
            group(arrange.into_iter().flatten().collect()),
            self.space_group(id),
            group(vec![
                copy_entry(tab.url()),
                entry(Icon::X, "Close tab", Action::Close(id)),
            ]),
        ]
    }

    fn favorite_groups(&self, index: usize) -> Vec<Group> {
        let Some(link) = self.current().favorites.get(index) else {
            return Vec::new();
        };
        let page = session::Pane {
            url: link.url.clone(),
            title: link.title.clone(),
        };
        vec![
            group(vec![
                split_entry(Action::Split(Partner::Page(page))),
                copy_entry(&link.url),
            ]),
            group(vec![
                entry(Icon::Pencil, "Edit bookmark…", Action::EditFavorite(index)),
                entry(
                    Icon::Star,
                    "Remove from Favorites",
                    Action::RemoveFavorite(index),
                ),
            ]),
        ]
    }

    fn profile_group(&self, space: &Space) -> Group {
        let entries = self
            .named_profiles()
            .into_iter()
            .map(|profile| {
                let icon = match profile.id == space.profile {
                    true => Icon::Check,
                    false => Icon::User,
                };
                let label = profile.name().to_owned();
                entry(icon, label, Action::AssignProfile(space.id, profile.id))
            })
            .collect();
        Group {
            heading: Some(self.text(Text::Profile)),
            entries,
        }
    }

    fn space_groups(&self, id: SpaceId) -> Vec<Group> {
        let Some(space) = self.store.session.space(id) else {
            return Vec::new();
        };
        if self.store.role().private() {
            return Vec::new();
        }
        let manage = entry(
            Icon::Settings,
            self.text(Text::ManageProfiles),
            Action::ManageProfiles,
        );
        vec![self.profile_group(space), group(vec![manage])]
    }

    fn groups(&self, target: Target, cx: &App) -> Vec<Group> {
        let groups = match target {
            Target::Tab(id) => self.tab_groups(id, cx),
            Target::Favorite(index) => self.favorite_groups(index),
            Target::Space(id) => self.space_groups(id),
        };
        groups
            .into_iter()
            .filter(|group| !group.entries.is_empty())
            .collect()
    }

    fn perform(&mut self, action: Action, window: &mut Window, cx: &mut Context<Self>) {
        match action {
            Action::Move(id, section) => self.move_tab(id, section, cx),
            Action::ToggleFavorite(id) => self.toggle_favorite_of(id, cx),
            Action::EditFavorite(index) => self.start_editing_favorite(index, window, cx),
            Action::RemoveFavorite(index) => self.remove_favorite(index, cx),
            Action::Duplicate(id) => self.duplicate(id, cx),
            Action::SplitActive => self.split(window, cx),
            Action::Split(partner) => self.split_with(partner, cx),
            Action::ToggleSound(id) => self.toggle_sound(id, cx),
            Action::MoveToSpace(id, space) => self.move_to_space(Item::Tab(id), space, cx),
            Action::AssignProfile(space, profile) => self.assign_profile(space, &profile, cx),
            Action::ManageProfiles => self.show_settings(SettingsSection::General, cx),
            Action::Copy(url) => cx.write_to_clipboard(ClipboardItem::new_string(url)),
            Action::Close(id) => self.close(id, cx),
        }
    }

    fn entry(&self, index: usize, entry: Entry, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let hover = colors.overlay;
        let action = entry.action;
        line(entry.icon, entry.label, colors)
            .id(("context-menu-entry", index))
            .cursor_pointer()
            .hover(move |row| row.bg(rgba(hover)))
            .on_click(cx.listener(move |browser, _, window, cx| {
                browser.dismiss_popover(window, cx);
                browser.perform(action.clone(), window, cx);
            }))
    }

    fn group(&self, position: usize, group: Group, cx: &mut Context<Self>) -> Stateful<Div> {
        let colors = self.colors();
        let entries = group
            .entries
            .into_iter()
            .enumerate()
            .map(|(index, entry)| self.entry(index, entry, cx));
        div()
            .id(("context-menu-group", position))
            .flex()
            .flex_col()
            .children(group.heading.map(|label| heading(label, colors).pb_1()))
            .children(entries)
    }

    pub(super) fn context_menu(
        &self,
        menu: &Menu,
        cx: &mut Context<Self>,
    ) -> Option<impl IntoElement> {
        let groups = self.groups(menu.target, cx);
        if groups.is_empty() {
            return None;
        }
        let colors = self.colors();
        let sections = groups
            .into_iter()
            .enumerate()
            .flat_map(|(position, group)| {
                let divider = (position > 0).then(|| divider(colors).into_any_element());
                let group = self.group(position, group, cx).into_any_element();
                divider.into_iter().chain([group])
            });
        let card = card(colors).children(sections.collect::<Vec<_>>());
        Some(
            anchored()
                .position(menu.position)
                .snap_to_window_with_margin(px(WINDOW_MARGIN_PX))
                .child(self.cutouts.see_through(PANEL_CORNER_PX).child(card)),
        )
    }
}
