package app.magpie.browser.address

import app.magpie.browser.favorite.Favorite
import app.magpie.browser.history.Visit

enum class Source { Site, Search, Bookmark, History }

data class Suggestion(val source: Source, val label: String, val detail: String, val target: String)

private const val RECENT_COUNT = 5
private const val PAGE_MATCH_COUNT = 4

private fun Favorite.suggestion() = Suggestion(Source.Bookmark, title.ifBlank { url }, url, url)

private fun Visit.suggestion() = Suggestion(Source.History, title.ifBlank { url }, url, url)

private fun typed(text: String): Suggestion =
    if (looksLikeAddress(text)) Suggestion(Source.Site, text, "Go to site", text) else Suggestion(Source.Search, text, "Search", text)

private fun Suggestion.matches(text: String): Boolean =
    label.contains(text, ignoreCase = true) || target.contains(text, ignoreCase = true)

fun suggestions(query: String, bookmarks: List<Favorite>, visits: List<Visit>): List<Suggestion> {
    val text = query.trim()
    if (text.isEmpty()) return visits.take(RECENT_COUNT).map { it.suggestion() }
    val pages = (bookmarks.asSequence().map { it.suggestion() } + visits.asSequence().map { it.suggestion() })
        .filter { it.matches(text) }
        .distinctBy(Suggestion::target)
        .take(PAGE_MATCH_COUNT)
    return listOf(typed(text)) + pages
}
