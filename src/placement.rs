use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use magpie_engine::file;

const FILE: &str = "window.json";

#[derive(Deserialize)]
struct UncheckedRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedRect")]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Windowed,
    Maximized,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub bounds: Rect,
    pub state: State,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    pub work_area: Rect,
    pub primary: bool,
}

impl TryFrom<UncheckedRect> for Rect {
    type Error = &'static str;

    fn try_from(unchecked: UncheckedRect) -> Result<Self, Self::Error> {
        let UncheckedRect {
            x,
            y,
            width,
            height,
        } = unchecked;
        let finite = [x, y, width, height].iter().all(|value| value.is_finite());
        if !finite || width <= 0.0 || height <= 0.0 {
            return Err("window bounds need a finite position and a positive size");
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
}

fn overlap(start: f32, length: f32, other_start: f32, other_length: f32) -> f32 {
    let end = (start + length).min(other_start + other_length);
    (end - start.max(other_start)).max(0.0)
}

impl Rect {
    fn shared_area(&self, other: &Rect) -> f32 {
        overlap(self.x, self.width, other.x, other.width)
            * overlap(self.y, self.height, other.y, other.height)
    }

    fn inside(self, area: Rect) -> Rect {
        let width = self.width.min(area.width);
        let height = self.height.min(area.height);
        Rect {
            x: self.x.clamp(area.x, area.x + area.width - width),
            y: self.y.clamp(area.y, area.y + area.height - height),
            width,
            height,
        }
    }
}

fn home_screen(bounds: &Rect, screens: &[Screen]) -> Option<usize> {
    let most_shown = screens
        .iter()
        .enumerate()
        .map(|(index, screen)| (index, bounds.shared_area(&screen.work_area)))
        .filter(|(_, area)| *area > 0.0)
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index);
    most_shown
        .or_else(|| screens.iter().position(|screen| screen.primary))
        .or_else(|| (!screens.is_empty()).then_some(0))
}

impl Placement {
    pub fn fit(self, screens: &[Screen]) -> Option<(usize, Placement)> {
        let index = home_screen(&self.bounds, screens)?;
        let bounds = self.bounds.inside(screens[index].work_area);
        Some((index, Placement { bounds, ..self }))
    }
}

pub fn load(folder: &Path) -> io::Result<Option<Placement>> {
    file::read(&folder.join(FILE))
}

pub fn save(folder: &Path, placement: &Placement) -> io::Result<()> {
    std::fs::create_dir_all(folder)?;
    file::save(&folder.join(FILE), placement)
}
