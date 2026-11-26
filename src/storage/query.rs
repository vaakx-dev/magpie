const HOST_PREFIX_SCORE: f64 = 4.0;
const WORD_START_SCORE: f64 = 3.0;
const SUBSTRING_SCORE: f64 = 1.0;
const URL_PREFIXES: [&str; 4] = ["https://www.", "http://www.", "https://", "http://"];

pub struct Query {
    terms: Vec<String>,
}

fn host_part(url: &str) -> &str {
    URL_PREFIXES
        .iter()
        .find_map(|prefix| url.strip_prefix(prefix))
        .unwrap_or(url)
}

fn starts_word(text: &str, term: &str) -> bool {
    text.match_indices(term).any(|(index, _)| {
        text[..index]
            .chars()
            .next_back()
            .is_none_or(|previous| !previous.is_alphanumeric())
    })
}

fn term_score(term: &str, title: &str, url: &str) -> Option<f64> {
    if host_part(url).starts_with(term) {
        return Some(HOST_PREFIX_SCORE);
    }
    if starts_word(title, term) {
        return Some(WORD_START_SCORE);
    }
    if title.contains(term) || url.contains(term) {
        return Some(SUBSTRING_SCORE);
    }
    None
}

impl Query {
    pub fn new(text: &str) -> Self {
        let terms = text.split_whitespace().map(str::to_lowercase).collect();
        Self { terms }
    }

    pub fn score(&self, title: &str, url: &str) -> Option<f64> {
        let title = title.to_lowercase();
        let url = url.to_lowercase();
        self.terms
            .iter()
            .map(|term| term_score(term, &title, &url))
            .try_fold(SUBSTRING_SCORE, |total, score| Some(total + score?))
    }

    pub fn matches(&self, title: &str, url: &str) -> bool {
        self.score(title, url).is_some()
    }
}
