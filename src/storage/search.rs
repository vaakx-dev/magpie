use std::collections::HashMap;

use super::archive::Archive;
use super::clock::Epoch;
use super::history::History;
use super::query::Query;
use super::session::{Session, SpaceId, TabId};

pub const MAX_RESULTS: usize = 12;
const FRECENCY_SCALE: f64 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    OpenTab,
    Pinned,
    Favorite,
    History,
    Archived,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult {
    pub source: Source,
    pub title: String,
    pub url: String,
    pub tab: Option<TabId>,
    pub space: Option<SpaceId>,
    pub score: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct OpenTab<'a> {
    pub id: TabId,
    pub title: &'a str,
    pub url: &'a str,
}

pub struct Sources<'a> {
    pub open_tabs: &'a [OpenTab<'a>],
    pub session: &'a Session,
    pub history: &'a History,
    pub archive: &'a Archive,
    pub now: Epoch,
}

struct Candidate<'a> {
    source: Source,
    title: &'a str,
    url: &'a str,
    tab: Option<TabId>,
    space: Option<SpaceId>,
    boost: f64,
}

impl Source {
    fn weight(self) -> f64 {
        match self {
            Source::OpenTab => 5.0,
            Source::Pinned => 4.0,
            Source::Favorite => 4.0,
            Source::History => 2.0,
            Source::Archived => 1.5,
        }
    }
}

fn open_tabs<'a>(sources: &Sources<'a>) -> impl Iterator<Item = Candidate<'a>> {
    sources.open_tabs.iter().map(|tab| Candidate {
        source: Source::OpenTab,
        title: tab.title,
        url: tab.url,
        tab: Some(tab.id),
        space: None,
        boost: 0.0,
    })
}

fn saved<'a>(sources: &Sources<'a>) -> impl Iterator<Item = Candidate<'a>> {
    sources.session.spaces.iter().flat_map(|space| {
        let favorites = space.favorites.iter().map(|link| Candidate {
            source: Source::Favorite,
            title: &link.title,
            url: &link.url,
            tab: None,
            space: Some(space.id),
            boost: 0.0,
        });
        let pinned = space.pinned_tabs().into_iter().map(|tab| Candidate {
            source: Source::Pinned,
            title: tab.title(),
            url: tab.url(),
            tab: Some(tab.id),
            space: Some(space.id),
            boost: 0.0,
        });
        favorites.chain(pinned)
    })
}

fn visited<'a>(sources: &Sources<'a>) -> impl Iterator<Item = Candidate<'a>> {
    let now = sources.now;
    sources.history.visits().map(move |visit| Candidate {
        source: Source::History,
        title: &visit.title,
        url: &visit.url,
        tab: None,
        space: None,
        boost: (1.0 + visit.frecency(now) / FRECENCY_SCALE).ln(),
    })
}

fn archived<'a>(sources: &Sources<'a>) -> impl Iterator<Item = Candidate<'a>> {
    sources.archive.list().map(|entry| Candidate {
        source: Source::Archived,
        title: entry.tab.title(),
        url: entry.tab.url(),
        tab: Some(entry.tab.id),
        space: Some(entry.space),
        boost: 0.0,
    })
}

fn rank(query: &Query, candidate: Candidate) -> Option<SearchResult> {
    let relevance = query.score(candidate.title, candidate.url)?;
    Some(SearchResult {
        source: candidate.source,
        title: candidate.title.to_owned(),
        url: candidate.url.to_owned(),
        tab: candidate.tab,
        space: candidate.space,
        score: relevance * candidate.source.weight() + candidate.boost,
    })
}

fn best_per_url(results: impl Iterator<Item = SearchResult>) -> Vec<SearchResult> {
    let mut best: HashMap<String, SearchResult> = HashMap::new();
    for result in results {
        let replaces = best
            .get(&result.url)
            .is_none_or(|current| result.score > current.score);
        if replaces {
            best.insert(result.url.clone(), result);
        }
    }
    best.into_values().collect()
}

fn candidates<'a>(sources: &'a Sources<'a>) -> impl Iterator<Item = Candidate<'a>> + 'a {
    open_tabs(sources)
        .chain(saved(sources))
        .chain(visited(sources))
        .chain(archived(sources))
}

pub fn search(query: &str, sources: &Sources) -> Vec<SearchResult> {
    let query = Query::new(query);
    let candidates = candidates(sources);
    let mut results = best_per_url(candidates.filter_map(|candidate| rank(&query, candidate)));
    results.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.url.cmp(&right.url))
    });
    results.truncate(MAX_RESULTS);
    results
}
