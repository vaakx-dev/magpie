use magpie::autocomplete::{completes, suggestions};

fn hosts(query: &str, known: &[&str]) -> Vec<String> {
    suggestions(query, known.iter().copied())
        .iter()
        .map(|url| url.host_str().unwrap().to_owned())
        .collect()
}

#[test]
fn prefixes_and_aliases_suggest_common_sites() {
    assert_eq!(hosts("yo", &[]), ["youtube.com"]);
    assert_eq!(hosts("git", &[]), ["github.com"]);
    assert_eq!(hosts("gh", &[]), ["github.com"]);
    assert_eq!(hosts("wi", &[]), ["wikipedia.org"]);
    assert_eq!(hosts("hn", &[]), ["news.ycombinator.com"]);
    assert_eq!(hosts("https://www.Twi", &[]), ["twitch.tv", "x.com"]);
    assert!(hosts("", &[]).is_empty());
    assert!(hosts("com", &[]).is_empty());
}

#[test]
fn domain_prefixes_rank_before_inner_labels_and_aliases() {
    assert_eq!(
        hosts("google", &[]),
        ["google.com", "docs.google.com", "drive.google.com"]
    );
    assert_eq!(hosts("ma", &[]), ["maps.google.com", "gmail.com"]);
}

#[test]
fn sites_the_user_already_has_are_left_out() {
    let known = [
        "https://www.youtube.com/watch?v=abc",
        "https://github.com/rust-lang/rust",
    ];
    assert!(hosts("yo", &known).is_empty());
    assert!(hosts("gh", &known).is_empty());
    assert_eq!(
        hosts("g", &known),
        ["google.com", "gmail.com", "docs.google.com"]
    );
}

#[test]
fn completion_needs_the_query_to_start_the_host() {
    assert!(completes("you", "https://www.youtube.com/watch?v=abc"));
    assert!(completes("https://git", "https://github.com/"));
    assert!(!completes("tube", "https://www.youtube.com/"));
    assert!(!completes("", "https://github.com/"));
}
