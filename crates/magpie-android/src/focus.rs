use magpie_engine::focus::{Focus, Rule};
use url::Url;

pub fn rules(blocked: &str) -> Focus {
    blocked.lines().filter_map(Rule::parse).collect()
}

pub fn redirect(focus: &Focus, address: &str) -> Option<String> {
    let url = Url::parse(address).ok()?;
    focus.redirect(&url).map(String::from)
}
