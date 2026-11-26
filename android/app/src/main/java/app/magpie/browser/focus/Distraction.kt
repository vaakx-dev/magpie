package app.magpie.browser.focus

enum class Distraction(val key: String, val label: String, val hint: String, val blockedByDefault: Boolean) {
    YouTubeShorts(
        "hide_youtube_shorts",
        "Block YouTube Shorts",
        "Hides Shorts shelves and buttons, and opens Shorts as normal videos",
        blockedByDefault = true,
    ),
    YouTubePlayables(
        "hide_youtube_playables",
        "Hide YouTube Playables",
        "Hides game shelves on Home and in search",
        blockedByDefault = true,
    ),
    YouTubeMembersOnly(
        "hide_youtube_members_only",
        "Hide members-only videos",
        "Hides videos you can't watch without a channel membership",
        blockedByDefault = true,
    ),
    YouTubeRecommendations(
        "hide_youtube_recommendations",
        "Hide YouTube recommendations",
        "Hides the Home feed and Up next. Search and Subscriptions stay",
        blockedByDefault = false,
    ),
    YouTubeComments(
        "hide_youtube_comments",
        "Hide YouTube comments",
        "Hides comment sections under videos",
        blockedByDefault = false,
    ),
    InstagramReels(
        "hide_instagram_reels",
        "Block Instagram Reels and Explore",
        "Hides the Reels and Explore tabs, feeds and Reels in your feed",
        blockedByDefault = false,
    ),
    FacebookReels(
        "hide_facebook_reels",
        "Block Facebook Reels and Watch",
        "Hides the Stories and Reels tray, Reels in your feed and the Watch feed",
        blockedByDefault = false,
    ),
    XForYou(
        "hide_x_for_you",
        "Hide X For you and trends",
        "Hides the For you timeline, trends and who to follow on x.com",
        blockedByDefault = false,
    ),
    TikTok(
        "block_tiktok",
        "Block TikTok feeds",
        "Hides the For You and Explore feeds on tiktok.com",
        blockedByDefault = false,
    ),
    RedditPopular(
        "hide_reddit_popular",
        "Hide Reddit Popular and All",
        "Hides the Popular, All and trending feeds and links",
        blockedByDefault = false,
    ),
}
