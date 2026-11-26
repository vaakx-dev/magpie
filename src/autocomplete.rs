use std::collections::HashSet;

use url::Url;

struct Site {
    domain: &'static str,
    aliases: &'static [&'static str],
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Fit {
    Domain,
    Label,
    Alias,
}

const MAX_SUGGESTIONS: usize = 3;
const SCHEMES: [&str; 2] = ["https://", "http://"];
const WWW: &str = "www.";

const SITES: [Site; 24] = [
    Site {
        domain: "google.com",
        aliases: &[],
    },
    Site {
        domain: "youtube.com",
        aliases: &["yt"],
    },
    Site {
        domain: "github.com",
        aliases: &["gh"],
    },
    Site {
        domain: "wikipedia.org",
        aliases: &["wiki"],
    },
    Site {
        domain: "reddit.com",
        aliases: &[],
    },
    Site {
        domain: "gmail.com",
        aliases: &["mail"],
    },
    Site {
        domain: "docs.google.com",
        aliases: &["gdocs"],
    },
    Site {
        domain: "drive.google.com",
        aliases: &["gdrive"],
    },
    Site {
        domain: "maps.google.com",
        aliases: &["gmaps"],
    },
    Site {
        domain: "news.ycombinator.com",
        aliases: &["hn", "hacker news"],
    },
    Site {
        domain: "x.com",
        aliases: &["twitter"],
    },
    Site {
        domain: "chatgpt.com",
        aliases: &["openai"],
    },
    Site {
        domain: "claude.ai",
        aliases: &["anthropic"],
    },
    Site {
        domain: "amazon.com",
        aliases: &[],
    },
    Site {
        domain: "netflix.com",
        aliases: &[],
    },
    Site {
        domain: "twitch.tv",
        aliases: &[],
    },
    Site {
        domain: "spotify.com",
        aliases: &[],
    },
    Site {
        domain: "stackoverflow.com",
        aliases: &["so"],
    },
    Site {
        domain: "linkedin.com",
        aliases: &[],
    },
    Site {
        domain: "duckduckgo.com",
        aliases: &["ddg"],
    },
    Site {
        domain: "instagram.com",
        aliases: &["ig"],
    },
    Site {
        domain: "facebook.com",
        aliases: &["fb"],
    },
    Site {
        domain: "outlook.com",
        aliases: &["hotmail"],
    },
    Site {
        domain: "bing.com",
        aliases: &[],
    },
];

fn typed(query: &str) -> String {
    let query = query.trim().to_lowercase();
    let unschemed = SCHEMES
        .iter()
        .find_map(|scheme| query.strip_prefix(scheme))
        .unwrap_or(query.as_str());
    unschemed.trim_start_matches(WWW).to_owned()
}

fn host(url: &str) -> Option<String> {
    let url = Url::parse(url).ok()?;
    Some(url.host_str()?.trim_start_matches(WWW).to_owned())
}

impl Site {
    fn inner_labels(&self) -> impl Iterator<Item = &'static str> {
        let (name, _top_level) = self.domain.rsplit_once('.').unwrap_or((self.domain, ""));
        name.split('.').skip(1)
    }

    fn fit(&self, query: &str) -> Option<Fit> {
        if self.domain.starts_with(query) {
            return Some(Fit::Domain);
        }
        if self.inner_labels().any(|label| label.starts_with(query)) {
            return Some(Fit::Label);
        }
        let aliased = self.aliases.iter().any(|alias| alias.starts_with(query));
        aliased.then_some(Fit::Alias)
    }

    fn url(&self) -> Url {
        Url::parse(&format!("https://{}/", self.domain)).expect("curated domains are valid hosts")
    }
}

pub fn completes(query: &str, url: &str) -> bool {
    let query = typed(query);
    !query.is_empty() && host(url).is_some_and(|host| host.starts_with(&query))
}

pub fn suggestions<'a>(query: &str, known: impl IntoIterator<Item = &'a str>) -> Vec<Url> {
    let query = typed(query);
    if query.is_empty() {
        return Vec::new();
    }
    let known: HashSet<String> = known.into_iter().filter_map(host).collect();
    let mut fits: Vec<(Fit, usize, &Site)> = SITES
        .iter()
        .enumerate()
        .filter(|(_, site)| !known.contains(site.domain))
        .filter_map(|(rank, site)| Some((site.fit(&query)?, rank, site)))
        .collect();
    fits.sort_unstable_by_key(|(fit, rank, _)| (*fit, *rank));
    fits.into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(_, _, site)| site.url())
        .collect()
}
