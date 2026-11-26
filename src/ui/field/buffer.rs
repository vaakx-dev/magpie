use std::{mem, ops::Range};

use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Home,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Word,
    Space,
    Symbol,
}

pub struct Buffer {
    content: String,
    anchor: usize,
    head: usize,
}

fn class(character: char) -> Class {
    match character {
        character if character.is_alphanumeric() || character == '_' => Class::Word,
        character if character.is_whitespace() => Class::Space,
        _ => Class::Symbol,
    }
}

impl Buffer {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_owned(),
            anchor: 0,
            head: content.len(),
        }
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn cursor(&self) -> usize {
        self.head
    }

    pub fn reversed(&self) -> bool {
        self.head < self.anchor
    }

    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.head)..self.anchor.max(self.head)
    }

    pub fn selected(&self) -> &str {
        &self.content[self.selection()]
    }

    pub fn move_to(&mut self, offset: usize) {
        self.anchor = offset;
        self.head = offset;
    }

    pub fn select_to(&mut self, offset: usize) {
        self.head = offset;
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.head = self.content.len();
    }

    pub fn select_word_at(&mut self, offset: usize) {
        let Some(word) = self.class_around(offset) else {
            return;
        };
        let start = self.content[..offset]
            .char_indices()
            .rev()
            .take_while(|(_, character)| class(*character) == word)
            .last()
            .map_or(offset, |(index, _)| index);
        let end = self.content[offset..]
            .char_indices()
            .find(|(_, character)| class(*character) != word)
            .map_or(self.content.len(), |(index, _)| offset + index);
        self.anchor = start;
        self.head = end;
    }

    pub fn move_by(&mut self, motion: Motion) {
        let selection = self.selection();
        let target = match (motion, selection.is_empty()) {
            (Motion::Left, false) => selection.start,
            (Motion::Right, false) => selection.end,
            _ => self.target(motion),
        };
        self.move_to(target);
    }

    pub fn select_by(&mut self, motion: Motion) {
        self.select_to(self.target(motion));
    }

    pub fn replace(&mut self, range: Range<usize>, text: &str) {
        self.content.replace_range(range.clone(), text);
        self.move_to(range.start + text.len());
    }

    pub fn take(&mut self) -> Zeroizing<String> {
        self.move_to(0);
        Zeroizing::new(mem::take(&mut self.content))
    }

    pub fn insert(&mut self, text: &str) {
        self.replace(self.selection(), text);
    }

    pub fn backspace(&mut self) {
        let selection = self.selection();
        let range = match selection.is_empty() {
            true => self.previous_boundary(self.head)..self.head,
            false => selection,
        };
        self.replace(range, "");
    }

    pub fn delete(&mut self) {
        let selection = self.selection();
        let range = match selection.is_empty() {
            true => self.head..self.next_boundary(self.head),
            false => selection,
        };
        self.replace(range, "");
    }

    fn class_around(&self, offset: usize) -> Option<Class> {
        let after = self.content[offset..].chars().next();
        let before = self.content[..offset].chars().next_back();
        after.or(before).map(class)
    }

    fn target(&self, motion: Motion) -> usize {
        match motion {
            Motion::Left => self.previous_boundary(self.head),
            Motion::Right => self.next_boundary(self.head),
            Motion::Home => 0,
            Motion::End => self.content.len(),
        }
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content[..offset]
            .char_indices()
            .next_back()
            .map_or(0, |(index, _)| index)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content[offset..]
            .chars()
            .next()
            .map_or(offset, |character| offset + character.len_utf8())
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.content.zeroize();
    }
}
