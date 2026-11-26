use serde::Deserialize;

use super::session::{FolderId, Pinned, Section, Session, Space, SpaceId, Tab, TabId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Item {
    Tab(TabId),
    Folder(FolderId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spot {
    Before(Item),
    After(Item),
    Into(FolderId),
    Top(Section),
}

impl Pinned {
    fn is(&self, item: Item) -> bool {
        match (self, item) {
            (Pinned::Tab(tab), Item::Tab(id)) => tab.id == id,
            (Pinned::Folder(folder), Item::Folder(id)) => folder.id == id,
            _ => false,
        }
    }

    pub fn into_tab(self) -> Option<Tab> {
        match self {
            Pinned::Tab(tab) => Some(tab),
            Pinned::Folder(_) => None,
        }
    }
}

fn container(items: &mut Vec<Pinned>, item: Item) -> Option<(&mut Vec<Pinned>, usize)> {
    if let Some(index) = items.iter().position(|entry| entry.is(item)) {
        return Some((items, index));
    }
    items.iter_mut().find_map(|entry| match entry {
        Pinned::Folder(folder) => container(&mut folder.items, item),
        Pinned::Tab(_) => None,
    })
}

fn holds(items: &[Pinned], item: Item) -> bool {
    items.iter().any(|entry| {
        entry.is(item) || matches!(entry, Pinned::Folder(folder) if holds(&folder.items, item))
    })
}

impl Space {
    fn pins(&self, item: Item) -> bool {
        holds(&self.pinned, item)
    }

    fn pins_spot(&self, spot: Spot) -> bool {
        match spot {
            Spot::Before(item) | Spot::After(item) => self.pins(item),
            Spot::Into(folder) => self.pins(Item::Folder(folder)),
            Spot::Top(section) => section == Section::Pinned,
        }
    }

    fn today_index(&self, item: Item) -> Option<usize> {
        match item {
            Item::Tab(id) => self.today.iter().position(|tab| tab.id == id),
            Item::Folder(_) => None,
        }
    }

    pub fn take(&mut self, item: Item) -> Option<(Section, Pinned)> {
        if let Some(index) = self.today_index(item) {
            return Some((Section::Today, Pinned::Tab(self.today.remove(index))));
        }
        let (items, index) = container(&mut self.pinned, item)?;
        Some((Section::Pinned, items.remove(index)))
    }

    fn insert_beside(&mut self, item: Item, entry: Pinned, offset: usize) -> Option<()> {
        match self.today_index(item) {
            Some(index) => self.today.insert(index + offset, entry.into_tab()?),
            None => {
                let (items, index) = container(&mut self.pinned, item)?;
                items.insert(index + offset, entry);
            }
        }
        Some(())
    }

    fn insert(&mut self, entry: Pinned, spot: Spot) -> Option<()> {
        match spot {
            Spot::Before(item) => self.insert_beside(item, entry, 0)?,
            Spot::After(item) => self.insert_beside(item, entry, 1)?,
            Spot::Into(folder) => self.folder_mut(folder)?.items.push(entry),
            Spot::Top(Section::Pinned) => self.pinned.insert(0, entry),
            Spot::Top(Section::Today) => self.today.insert(0, entry.into_tab()?),
        }
        Some(())
    }

    fn receive(&mut self, section: Section, entry: Pinned) {
        match (section, entry) {
            (Section::Today, Pinned::Tab(tab)) => self.today.insert(0, tab),
            (_, entry) => self.pinned.push(entry),
        }
    }

    fn settle_active(&mut self) {
        if self.active.is_some_and(|id| self.tab(id).is_none()) {
            self.active = self.today.first().map(|tab| tab.id);
        }
    }

    pub fn move_favorite(&mut self, from: usize, slot: usize) -> bool {
        let count = self.favorites.len();
        if from >= count || slot > count {
            return false;
        }
        let to = match slot > from {
            true => slot - 1,
            false => slot,
        };
        if to == from {
            return false;
        }
        let favorite = self.favorites.remove(from);
        self.favorites.insert(to, favorite);
        true
    }

    pub fn arrange(&mut self, item: Item, spot: Spot) -> bool {
        let mut arranged = self.clone();
        let placed = arranged
            .take(item)
            .and_then(|(_, entry)| arranged.insert(entry, spot))
            .is_some();
        if placed {
            *self = arranged;
        }
        placed
    }
}

impl Session {
    pub fn move_favorite(&mut self, space: SpaceId, url: &str, slot: usize) -> bool {
        let Some(space) = self.space_mut(space) else {
            return false;
        };
        let Some(from) = space.favorites.iter().position(|link| link.url == url) else {
            return false;
        };
        let slot = slot.min(space.favorites.len());
        space.move_favorite(from, slot)
    }

    pub fn arrange_pinned(&mut self, item: Item, spot: Spot) -> bool {
        self.spaces
            .iter_mut()
            .find(|space| space.pins(item))
            .is_some_and(|space| space.pins_spot(spot) && space.arrange(item, spot))
    }

    pub fn move_to_space(&mut self, item: Item, to: SpaceId) -> bool {
        let Some(target) = self.spaces.iter().position(|space| space.id == to) else {
            return false;
        };
        let taken = self
            .spaces
            .iter_mut()
            .enumerate()
            .filter(|(index, _)| *index != target)
            .find_map(|(_, space)| {
                let taken = space.take(item)?;
                space.settle_active();
                Some(taken)
            });
        let Some((section, entry)) = taken else {
            return false;
        };
        self.spaces[target].receive(section, entry);
        true
    }
}
