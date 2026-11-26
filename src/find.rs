#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Matches {
    pub active: Option<u32>,
    pub count: u32,
}

impl Matches {
    pub fn new(active_position: i32, count: i32) -> Self {
        Self {
            active: u32::try_from(active_position)
                .ok()
                .filter(|position| *position > 0),
            count: u32::try_from(count).unwrap_or_default(),
        }
    }

    pub fn label(&self) -> String {
        match (self.count, self.active) {
            (0, _) => "No matches".to_owned(),
            (count, Some(active)) => format!("{active} / {count}"),
            (count, None) => format!("0 / {count}"),
        }
    }
}
