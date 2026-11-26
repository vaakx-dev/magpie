package app.magpie.browser.address

import app.magpie.browser.favorite.Favorite
import app.magpie.browser.history.Visit
import org.junit.Assert.assertEquals
import org.junit.Test

class SuggestionsTest {
    private val bookmarks = listOf(Favorite("https://news.example.org/", "Morning news"), Favorite("https://maps.example.com/", "Maps"))
    private val visits = listOf(
        Visit("https://news.example.org/", "Morning news"),
        Visit("https://example.org/magpies", "All about magpies"),
        Visit("https://example.org/crows", ""),
    )

    @Test
    fun anEmptyQueryShowsRecentHistory() {
        val recent = suggestions("  ", bookmarks, visits)

        assertEquals(visits.map { it.url }, recent.map { it.target })
        assertEquals("https://example.org/crows", recent.last().label)
    }

    @Test
    fun typedTextComesFirstThenMatchingBookmarksAndHistoryOncePerPage() {
        val matches = suggestions(" news ", bookmarks, visits)

        assertEquals(
            listOf(Suggestion(Source.Search, "news", "Search", "news"), Suggestion(Source.Bookmark, "Morning news", "https://news.example.org/", "https://news.example.org/")),
            matches,
        )
    }

    @Test
    fun anAddressIsOfferedAsASiteAndMatchesUrls() {
        val matches = suggestions("example.org/magpies", bookmarks, visits)

        assertEquals(listOf(Source.Site, Source.History), matches.map { it.source })
        assertEquals("All about magpies", matches.last().label)
    }
}
