mod rule;
mod shorts;
mod site;

use std::collections::BTreeSet;

use serde::Serialize;
use url::Url;

pub use rule::Rule;
pub use site::Site;

use crate::blocker::stylesheet;

const SCRIPT: &str = include_str!("focus.js");
const APPLY: &str = "__magpieFocus";

#[derive(Serialize)]
struct Sheet {
    domains: &'static [&'static str],
    css: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Focus {
    blocked: BTreeSet<Rule>,
}

impl Default for Focus {
    fn default() -> Self {
        Self {
            blocked: Rule::ALL
                .into_iter()
                .filter(|rule| rule.blocked_by_default())
                .collect(),
        }
    }
}

impl FromIterator<Rule> for Focus {
    fn from_iter<Rules: IntoIterator<Item = Rule>>(rules: Rules) -> Self {
        Self {
            blocked: rules.into_iter().collect(),
        }
    }
}

impl Focus {
    pub fn blocks(&self, rule: Rule) -> bool {
        self.blocked.contains(&rule)
    }

    pub fn block(&mut self, rule: Rule) {
        self.blocked.insert(rule);
    }

    pub fn allow(&mut self, rule: Rule) {
        self.blocked.remove(&rule);
    }

    pub fn stylesheet(&self, site: Site) -> String {
        let selectors: Vec<String> = self
            .blocked
            .iter()
            .filter(|rule| rule.site() == site)
            .flat_map(|rule| rule.selectors())
            .map(|selector| (*selector).to_owned())
            .collect();
        stylesheet(&selectors)
    }

    pub fn redirect(&self, url: &Url) -> Option<Url> {
        self.blocks(Rule::YouTubeShorts)
            .then(|| shorts::watch(url))
            .flatten()
    }

    pub fn script(&self) -> String {
        format!("({SCRIPT})({}, {APPLY:?});", self.sheets())
    }

    pub fn update(&self) -> String {
        format!("window.{APPLY}?.({});", self.sheets())
    }

    fn sheets(&self) -> String {
        let sheets: Vec<Sheet> = Site::ALL
            .into_iter()
            .map(|site| Sheet {
                domains: site.domains(),
                css: self.stylesheet(site),
            })
            .collect();
        serde_json::to_string(&sheets).expect("the focus sheets serialize")
    }
}
