use serde::Serialize;

use super::{category::Category, config::Config};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Switch {
    key: &'static str,
    label: &'static str,
    description: &'static str,
    color: &'static str,
    on: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct View {
    enabled: bool,
    show_skip_notice: bool,
    categories: Vec<Switch>,
    channels: Vec<String>,
}

fn switch(config: &Config, category: Category) -> Switch {
    Switch {
        key: category.name(),
        label: category.label(),
        description: category.description(),
        color: category.color(),
        on: config.categories.contains(category),
    }
}

impl From<&Config> for View {
    fn from(config: &Config) -> Self {
        Self {
            enabled: config.enabled,
            show_skip_notice: config.show_skip_notice,
            categories: Category::ALL
                .into_iter()
                .map(|category| switch(config, category))
                .collect(),
            channels: config.channel_allowlist.clone(),
        }
    }
}
