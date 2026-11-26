use super::category::{Categories, Category};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub enabled: bool,
    pub categories: Categories,
    pub show_skip_notice: bool,
    pub channel_allowlist: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Channel {
    pub id: Option<String>,
    pub name: Option<String>,
}

const ON: &str = "On";
const OFF: &str = "Off";
const ENABLED: &str = "enabled";
const SHOW_SKIP_NOTICE: &str = "show_skip_notice";
const ALLOWED_CHANNEL: &str = "allowed_channel";

fn switch(on: bool) -> &'static str {
    if on { ON } else { OFF }
}

fn parse_switch(text: &str) -> Option<bool> {
    match text {
        text if text.eq_ignore_ascii_case(ON) => Some(true),
        text if text.eq_ignore_ascii_case(OFF) => Some(false),
        _ => None,
    }
}

fn line(key: &str, value: &str) -> String {
    format!("{key} = {value}\n")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            categories: Categories::default(),
            show_skip_notice: true,
            channel_allowlist: Vec::new(),
        }
    }
}

impl Channel {
    fn names(&self) -> impl Iterator<Item = &str> {
        [&self.id, &self.name]
            .into_iter()
            .flatten()
            .map(String::as_str)
    }
}

impl Config {
    pub fn parse(text: &str) -> Self {
        text.lines().filter_map(|line| line.split_once('=')).fold(
            Self::default(),
            |mut config, (key, value)| {
                config.set(key.trim(), value.trim());
                config
            },
        )
    }

    fn set_category(&mut self, category: Category, value: &str) {
        match parse_switch(value) {
            Some(true) => self.categories.insert(category),
            Some(false) => self.categories.remove(category),
            None => {}
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        match (key, Category::parse(key)) {
            (ENABLED, _) => self.enabled = parse_switch(value).unwrap_or(self.enabled),
            (SHOW_SKIP_NOTICE, _) => {
                self.show_skip_notice = parse_switch(value).unwrap_or(self.show_skip_notice)
            }
            (ALLOWED_CHANNEL, _) if !value.is_empty() => {
                self.channel_allowlist.push(value.to_owned())
            }
            (_, Some(category)) => self.set_category(category, value),
            _ => {}
        }
    }

    pub fn resume(&mut self, channel: &str) {
        self.channel_allowlist.retain(|allowed| allowed != channel);
    }

    pub fn text(&self) -> String {
        let switches = [
            line(ENABLED, switch(self.enabled)),
            line(SHOW_SKIP_NOTICE, switch(self.show_skip_notice)),
        ];
        let categories = Category::ALL
            .into_iter()
            .map(|category| line(category.name(), switch(self.categories.contains(category))));
        let channels = self
            .channel_allowlist
            .iter()
            .map(|channel| line(ALLOWED_CHANNEL, channel));
        switches
            .into_iter()
            .chain(categories)
            .chain(channels)
            .collect()
    }

    pub fn exempts(&self, channel: &Channel) -> bool {
        channel.names().any(|name| {
            self.channel_allowlist
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(name))
        })
    }
}
