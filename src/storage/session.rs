use serde::{Deserialize, Serialize};

use super::{
    arrange::Item,
    clock::Epoch,
    profile::{ProfileId, Profiles},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TabId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpaceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FolderId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    Pinned,
    Today,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Index(usize),
    Last,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Pane {
    pub url: String,
    pub title: String,
}

#[derive(Deserialize)]
struct UncheckedTab {
    id: TabId,
    #[serde(default)]
    name: Option<String>,
    panes: Vec<Pane>,
    focused: usize,
    last_active: Epoch,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedTab")]
pub struct Tab {
    pub id: TabId,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    panes: Vec<Pane>,
    focused: usize,
    pub last_active: Epoch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub collapsed: bool,
    pub items: Vec<Pinned>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Pinned {
    Tab(Tab),
    Folder(Folder),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Space {
    pub id: SpaceId,
    pub name: String,
    pub theme: String,
    #[serde(default, skip_serializing_if = "ProfileId::built_in")]
    pub profile: ProfileId,
    pub favorites: Vec<Link>,
    pub pinned: Vec<Pinned>,
    pub today: Vec<Tab>,
    pub active: Option<TabId>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub spaces: Vec<Space>,
    pub active: Option<SpaceId>,
}

fn trimmed(name: &str) -> Option<&str> {
    Some(name.trim()).filter(|name| !name.is_empty())
}

fn rename(current: &mut String, name: &str) -> bool {
    match trimmed(name) {
        Some(name) if name != current => {
            *current = name.to_owned();
            true
        }
        _ => false,
    }
}

impl TryFrom<UncheckedTab> for Tab {
    type Error = &'static str;

    fn try_from(unchecked: UncheckedTab) -> Result<Self, Self::Error> {
        if unchecked.focused >= unchecked.panes.len() {
            return Err("focused pane is out of range");
        }
        Ok(Self {
            id: unchecked.id,
            name: unchecked.name,
            panes: unchecked.panes,
            focused: unchecked.focused,
            last_active: unchecked.last_active,
        })
    }
}

impl Tab {
    pub fn new(id: TabId, pane: Pane, last_active: Epoch) -> Self {
        Self {
            id,
            name: None,
            panes: vec![pane],
            focused: 0,
            last_active,
        }
    }

    pub fn panes(&self) -> &[Pane] {
        &self.panes
    }

    pub fn focused(&self) -> &Pane {
        &self.panes[self.focused]
    }

    pub fn focused_index(&self) -> usize {
        self.focused
    }

    pub fn pane_mut(&mut self, index: usize) -> Option<&mut Pane> {
        self.panes.get_mut(index)
    }

    pub fn detach(&mut self, index: usize) -> Option<Pane> {
        if self.panes.len() < 2 || index >= self.panes.len() {
            return None;
        }
        self.focused = 0;
        Some(self.panes.remove(index))
    }

    pub fn url(&self) -> &str {
        &self.focused().url
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn title(&self) -> &str {
        self.name().unwrap_or(&self.focused().title)
    }

    pub fn rename(&mut self, name: &str) -> bool {
        let name = trimmed(name).map(str::to_owned);
        let changed = self.name != name;
        self.name = name;
        changed
    }

    pub fn duplicate(&self, id: TabId, now: Epoch) -> Self {
        Self {
            id,
            last_active: now,
            ..self.clone()
        }
    }

    pub fn split(&mut self, pane: Pane) {
        self.panes.push(pane);
        self.focused = self.panes.len() - 1;
    }

    pub fn focus(&mut self, index: usize) -> bool {
        let exists = index < self.panes.len();
        if exists {
            self.focused = index;
        }
        exists
    }
}

impl Folder {
    pub fn new(id: FolderId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            collapsed: false,
            items: Vec::new(),
        }
    }
}

fn collect_tabs<'a>(items: &'a [Pinned], tabs: &mut Vec<&'a Tab>) {
    for item in items {
        match item {
            Pinned::Tab(tab) => tabs.push(tab),
            Pinned::Folder(folder) => collect_tabs(&folder.items, tabs),
        }
    }
}

fn shift_pinned_ids(items: &mut [Pinned], offset: u64) {
    for item in items {
        match item {
            Pinned::Tab(tab) => tab.id = tab.id.shifted(offset),
            Pinned::Folder(folder) => shift_pinned_ids(&mut folder.items, offset),
        }
    }
}

fn largest_folder_id(items: &[Pinned]) -> u64 {
    items
        .iter()
        .map(|item| match item {
            Pinned::Folder(folder) => folder.id.0.max(largest_folder_id(&folder.items)),
            Pinned::Tab(_) => 0,
        })
        .max()
        .unwrap_or(0)
}

fn folder_in(items: &[Pinned], id: FolderId) -> Option<&Folder> {
    items.iter().find_map(|item| match item {
        Pinned::Folder(folder) => match folder.id == id {
            true => Some(folder),
            false => folder_in(&folder.items, id),
        },
        Pinned::Tab(_) => None,
    })
}

fn find_folder(items: &mut [Pinned], id: FolderId) -> Option<&mut Folder> {
    items.iter_mut().find_map(|item| match item {
        Pinned::Folder(folder) => match folder.id == id {
            true => Some(folder),
            false => find_folder(&mut folder.items, id),
        },
        Pinned::Tab(_) => None,
    })
}

fn find_tab(items: &mut [Pinned], id: TabId) -> Option<&mut Tab> {
    items.iter_mut().find_map(|item| match item {
        Pinned::Tab(tab) if tab.id == id => Some(tab),
        Pinned::Tab(_) => None,
        Pinned::Folder(folder) => find_tab(&mut folder.items, id),
    })
}

impl TabId {
    fn shifted(self, offset: u64) -> Self {
        Self(self.0 + offset)
    }
}

impl Space {
    fn shift_tab_ids(&mut self, offset: u64) {
        shift_pinned_ids(&mut self.pinned, offset);
        for tab in &mut self.today {
            tab.id = tab.id.shifted(offset);
        }
        self.active = self.active.map(|id| id.shifted(offset));
    }

    pub fn new(id: SpaceId, name: impl Into<String>, theme: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            theme: theme.into(),
            profile: ProfileId::default(),
            favorites: Vec::new(),
            pinned: Vec::new(),
            today: Vec::new(),
            active: None,
        }
    }

    pub fn pinned_tabs(&self) -> Vec<&Tab> {
        let mut tabs = Vec::new();
        collect_tabs(&self.pinned, &mut tabs);
        tabs
    }

    pub fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.pinned_tabs().into_iter().chain(&self.today)
    }

    pub fn tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs().find(|tab| tab.id == id)
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tab(self.active?)
    }

    pub fn cycled(&self, direction: Direction) -> Option<TabId> {
        let ids: Vec<TabId> = self.tabs().map(|tab| tab.id).collect();
        let last = ids.len().checked_sub(1)?;
        let current = ids.iter().position(|id| Some(*id) == self.active);
        let next = match (direction, current) {
            (Direction::Forward, Some(index)) if index == last => 0,
            (Direction::Forward, Some(index)) => index + 1,
            (Direction::Forward, None) => 0,
            (Direction::Backward, Some(0) | None) => last,
            (Direction::Backward, Some(index)) => index - 1,
        };
        Some(ids[next])
    }

    pub fn tab_at(&self, position: Position) -> Option<TabId> {
        let tab = match position {
            Position::Index(index) => self.tabs().nth(index),
            Position::Last => self.tabs().last(),
        };
        tab.map(|tab| tab.id)
    }

    pub fn section(&self, id: TabId) -> Option<Section> {
        let today = self.today.iter().any(|tab| tab.id == id);
        let pinned = self.pinned_tabs().iter().any(|tab| tab.id == id);
        match (today, pinned) {
            (true, _) => Some(Section::Today),
            (_, true) => Some(Section::Pinned),
            _ => None,
        }
    }

    pub fn remove(&mut self, id: TabId) -> Option<Tab> {
        let (_, entry) = self.take(Item::Tab(id))?;
        entry.into_tab()
    }

    pub fn folder(&self, id: FolderId) -> Option<&Folder> {
        folder_in(&self.pinned, id)
    }

    pub fn folder_mut(&mut self, id: FolderId) -> Option<&mut Folder> {
        find_folder(&mut self.pinned, id)
    }

    pub fn tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        match self.today.iter_mut().find(|tab| tab.id == id) {
            Some(tab) => Some(tab),
            None => find_tab(&mut self.pinned, id),
        }
    }

    pub fn activate(&mut self, id: TabId, now: Epoch) -> bool {
        let Some(tab) = self.tab_mut(id) else {
            return false;
        };
        tab.last_active = now;
        self.active = Some(id);
        true
    }

    pub fn reopen(&mut self, mut tab: Tab, now: Epoch) -> SpaceId {
        tab.last_active = now;
        self.active = Some(tab.id);
        self.today.insert(0, tab);
        self.id
    }
}

impl Session {
    pub fn space(&self, id: SpaceId) -> Option<&Space> {
        self.spaces.iter().find(|space| space.id == id)
    }

    pub fn space_mut(&mut self, id: SpaceId) -> Option<&mut Space> {
        self.spaces.iter_mut().find(|space| space.id == id)
    }

    pub fn active_space(&self) -> Option<&Space> {
        self.space(self.active?)
    }

    pub fn active_space_mut(&mut self) -> Option<&mut Space> {
        let id = self.active?;
        self.space_mut(id)
    }

    pub fn home(&mut self, preferred: SpaceId) -> Option<&mut Space> {
        let id = self
            .space(preferred)
            .map(|space| space.id)
            .or(self.active)
            .or(self.spaces.first().map(|space| space.id))?;
        self.space_mut(id)
    }

    pub fn space_of(&self, tab: TabId) -> Option<SpaceId> {
        self.spaces
            .iter()
            .find(|space| space.tab(tab).is_some())
            .map(|space| space.id)
    }

    pub fn profile_of(&self, tab: TabId) -> Option<&ProfileId> {
        self.spaces
            .iter()
            .find(|space| space.tab(tab).is_some())
            .map(|space| &space.profile)
    }

    pub fn assign_profile(&mut self, space: SpaceId, profile: &ProfileId) -> bool {
        self.space_mut(space)
            .filter(|space| &space.profile != profile)
            .map(|space| space.profile = profile.clone())
            .is_some()
    }

    pub fn keep_profiles(&mut self, profiles: &Profiles) -> bool {
        let orphaned = self
            .spaces
            .iter_mut()
            .filter(|space| !profiles.contains(&space.profile));
        orphaned
            .map(|space| space.profile = ProfileId::default())
            .count()
            > 0
    }

    pub fn next_space_id(&self) -> SpaceId {
        let largest = self.spaces.iter().map(|space| space.id.0).max();
        SpaceId(largest.map_or(1, |id| id + 1))
    }

    pub fn edit_favorite(&mut self, space: SpaceId, url: &str, edited: Link) -> bool {
        let Some(space) = self.space_mut(space) else {
            return false;
        };
        let taken = edited.url != url && space.favorites.iter().any(|link| link.url == edited.url);
        let Some(link) = space.favorites.iter_mut().find(|link| link.url == url) else {
            return false;
        };
        if taken {
            return false;
        }
        let renamed = rename(&mut link.title, &edited.title);
        let moved = link.url != edited.url;
        link.url = edited.url;
        renamed || moved
    }

    pub fn rename_tab(&mut self, id: TabId, name: &str) -> bool {
        self.spaces
            .iter_mut()
            .find_map(|space| space.tab_mut(id))
            .is_some_and(|tab| tab.rename(name))
    }

    pub fn rename_folder(&mut self, id: FolderId, name: &str) -> bool {
        self.spaces
            .iter_mut()
            .find_map(|space| space.folder_mut(id))
            .is_some_and(|folder| rename(&mut folder.name, name))
    }

    pub fn shift_tab_ids(&mut self, offset: u64) {
        for space in &mut self.spaces {
            space.shift_tab_ids(offset);
        }
    }

    pub fn next_folder_id(&self) -> FolderId {
        let largest = self
            .spaces
            .iter()
            .map(|space| largest_folder_id(&space.pinned))
            .max()
            .unwrap_or(0);
        FolderId(largest + 1)
    }
}
