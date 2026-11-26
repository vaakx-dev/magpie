use super::site::Site;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    YouTubeShorts,
    YouTubePlayables,
    YouTubeMembersOnly,
    YouTubeRecommendations,
    YouTubeComments,
    InstagramReels,
    FacebookReels,
    XForYou,
    TikTok,
    RedditPopular,
}

const YOUTUBE_SHORTS: &[&str] = &[
    "ytd-reel-shelf-renderer",
    "ytd-rich-shelf-renderer[is-shorts]",
    "ytd-rich-section-renderer:has(ytd-rich-shelf-renderer[is-shorts])",
    "ytd-rich-item-renderer:has(a[href^=\"/shorts/\"])",
    "ytd-video-renderer:has(a[href^=\"/shorts/\"])",
    "ytd-grid-video-renderer:has(a[href^=\"/shorts/\"])",
    "grid-shelf-view-model:has(ytm-shorts-lockup-view-model)",
    "ytm-shorts-lockup-view-model",
    "ytm-shorts-lockup-view-model-v2",
    "ytd-guide-entry-renderer:has(a[title=\"Shorts\"])",
    "ytd-guide-entry-renderer:has(a[href^=\"/shorts\"])",
    "ytd-mini-guide-entry-renderer[aria-label=\"Shorts\"]",
    "ytd-mini-guide-entry-renderer:has(a[href^=\"/shorts\"])",
    "yt-tab-shape[tab-title=\"Shorts\"]",
    "ytm-reel-shelf-renderer",
    "ytm-rich-section-renderer:has(ytm-reel-shelf-renderer)",
    "ytm-rich-section-renderer:has(ytm-shorts-lockup-view-model)",
    "ytm-rich-item-renderer:has(a[href^=\"/shorts/\"])",
    "ytm-video-with-context-renderer:has(a[href^=\"/shorts/\"])",
    "ytm-compact-video-renderer:has(a[href^=\"/shorts/\"])",
    "ytm-pivot-bar-item-renderer:has(.pivot-shorts)",
];

const YOUTUBE_PLAYABLES: &[&str] = &[
    "ytd-rich-section-renderer:has(a[href^=\"/playables\"])",
    "ytd-rich-shelf-renderer:has(a[href^=\"/playables\"])",
    "ytd-shelf-renderer:has(a[href^=\"/playables\"])",
    "ytd-rich-item-renderer:has(ytd-mini-game-card-view-model)",
    "ytd-mini-game-card-view-model",
    "ytd-guide-entry-renderer:has(a[href^=\"/playables\"])",
    "ytm-rich-section-renderer:has(a[href^=\"/playables\"])",
    "ytm-rich-item-renderer:has(ytm-mini-game-card-view-model)",
    "ytm-mini-game-card-view-model",
];

const YOUTUBE_MEMBERS_ONLY: &[&str] = &[
    "ytd-rich-item-renderer:has(.badge-style-type-members-only)",
    "ytd-video-renderer:has(.badge-style-type-members-only)",
    "ytd-grid-video-renderer:has(.badge-style-type-members-only)",
    "ytd-compact-video-renderer:has(.badge-style-type-members-only)",
    "ytm-rich-item-renderer:has(.badge-style-type-members-only)",
    "ytm-video-with-context-renderer:has(.badge-style-type-members-only)",
    "ytm-compact-video-renderer:has(.badge-style-type-members-only)",
];

const YOUTUBE_RECOMMENDATIONS: &[&str] = &[
    "ytd-browse[page-subtype=\"home\"] ytd-rich-grid-renderer",
    "ytd-watch-flexy #related",
    "ytd-watch-next-secondary-results-renderer",
    ".ytp-endscreen-content",
    "html[data-magpie-path=\"/\"] ytm-browse ytm-rich-grid-renderer",
    "html[data-magpie-path=\"/\"] ytm-browse ytm-section-list-renderer",
    "ytm-item-section-renderer[section-identifier=\"related-items\"]",
];

const YOUTUBE_COMMENTS: &[&str] = &[
    "ytd-comments#comments",
    "ytd-engagement-panel-section-list-renderer[target-id=\"engagement-panel-comments-section\"]",
    "ytm-comment-section-renderer",
    "ytm-comments-entry-point-header-renderer",
    "ytm-item-section-renderer[section-identifier=\"comments-entry-point\"]",
    "ytm-engagement-panel-section-list-renderer:has(ytm-comment-section-renderer)",
];

const INSTAGRAM_REELS: &[&str] = &[
    "a[href=\"/reels/\"]",
    "a[href=\"/explore/\"]",
    "a[role=\"tab\"][href$=\"/reels/\"]",
    "article:has(a[href^=\"/reel/\"])",
    "html[data-magpie-path^=\"/reels\"] main",
    "html[data-magpie-path=\"/explore/\"] main",
];

const FACEBOOK_REELS: &[&str] = &[
    "[data-pagelet=\"Stories\"]",
    "div[data-pagelet^=\"FeedUnit\"]:has(a[href*=\"/reel/\"])",
    "a[href=\"/watch/\"]",
    "a[href=\"https://www.facebook.com/watch/\"]",
    "a[href^=\"/reel/?s=\"]",
    "a[aria-label=\"Reels\"]",
    "a[aria-label=\"Watch\"]",
    "html[data-magpie-path^=\"/watch\"] [role=\"main\"]",
    "html[data-magpie-path^=\"/reel\"] [role=\"main\"]",
];

const X_FOR_YOU: &[&str] = &[
    "[data-testid=\"trend\"]",
    "[data-testid=\"sidebarColumn\"] section:has([data-testid=\"trend\"])",
    "[data-testid=\"sidebarColumn\"] aside",
    "html[data-magpie-path=\"/home\"] [data-testid=\"primaryColumn\"] [role=\"tablist\"] [role=\"presentation\"]:first-child",
    "html[data-magpie-path=\"/home\"] [data-testid=\"primaryColumn\"]:has([role=\"tablist\"] [role=\"presentation\"]:first-child [aria-selected=\"true\"]) section",
    "html[data-magpie-path^=\"/explore\"] [data-testid=\"primaryColumn\"] section",
];

const TIKTOK: &[&str] = &[
    "[data-e2e=\"recommend-list-item-container\"]",
    "[data-e2e=\"explore-item-list\"]",
    "[data-e2e=\"nav-foryou\"]",
    "[data-e2e=\"nav-explore\"]",
    "#main-content-homepage_hot",
    "#main-content-explore_page",
    "#column-list-container",
    "html[data-magpie-path=\"/\"] main",
    "html[data-magpie-path^=\"/foryou\"] main",
    "html[data-magpie-path^=\"/explore\"] main",
];

const REDDIT_POPULAR: &[&str] = &[
    "a[href=\"/r/popular\"]",
    "a[href=\"/r/popular/\"]",
    "a[href=\"/r/all/\"]",
    "a[href$=\"reddit.com/r/popular/\"]",
    "a[href$=\"reddit.com/r/all/\"]",
    "faceplate-tracker[noun=\"popular\"]",
    "#reddit-trending-searches-partial-container",
    "html[data-magpie-path^=\"/r/popular\"] shreddit-feed",
    "html[data-magpie-path^=\"/r/all\"] shreddit-feed",
    "html[data-magpie-path^=\"/r/popular\"] #siteTable",
    "html[data-magpie-path^=\"/r/all\"] #siteTable",
];

impl Rule {
    pub const ALL: [Self; 10] = [
        Self::YouTubeShorts,
        Self::YouTubePlayables,
        Self::YouTubeMembersOnly,
        Self::YouTubeRecommendations,
        Self::YouTubeComments,
        Self::InstagramReels,
        Self::FacebookReels,
        Self::XForYou,
        Self::TikTok,
        Self::RedditPopular,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::YouTubeShorts => "hide_youtube_shorts",
            Self::YouTubePlayables => "hide_youtube_playables",
            Self::YouTubeMembersOnly => "hide_youtube_members_only",
            Self::YouTubeRecommendations => "hide_youtube_recommendations",
            Self::YouTubeComments => "hide_youtube_comments",
            Self::InstagramReels => "hide_instagram_reels",
            Self::FacebookReels => "hide_facebook_reels",
            Self::XForYou => "hide_x_for_you",
            Self::TikTok => "block_tiktok",
            Self::RedditPopular => "hide_reddit_popular",
        }
    }

    pub fn parse(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|rule| rule.key() == key)
    }

    pub fn blocked_by_default(self) -> bool {
        matches!(
            self,
            Self::YouTubeShorts | Self::YouTubePlayables | Self::YouTubeMembersOnly
        )
    }

    pub fn site(self) -> Site {
        match self {
            Self::YouTubeShorts
            | Self::YouTubePlayables
            | Self::YouTubeMembersOnly
            | Self::YouTubeRecommendations
            | Self::YouTubeComments => Site::YouTube,
            Self::InstagramReels => Site::Instagram,
            Self::FacebookReels => Site::Facebook,
            Self::XForYou => Site::X,
            Self::TikTok => Site::TikTok,
            Self::RedditPopular => Site::Reddit,
        }
    }

    pub(super) fn selectors(self) -> &'static [&'static str] {
        match self {
            Self::YouTubeShorts => YOUTUBE_SHORTS,
            Self::YouTubePlayables => YOUTUBE_PLAYABLES,
            Self::YouTubeMembersOnly => YOUTUBE_MEMBERS_ONLY,
            Self::YouTubeRecommendations => YOUTUBE_RECOMMENDATIONS,
            Self::YouTubeComments => YOUTUBE_COMMENTS,
            Self::InstagramReels => INSTAGRAM_REELS,
            Self::FacebookReels => FACEBOOK_REELS,
            Self::XForYou => X_FOR_YOU,
            Self::TikTok => TIKTOK,
            Self::RedditPopular => REDDIT_POPULAR,
        }
    }
}
