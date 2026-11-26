package app.magpie.browser.browser

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test

class SavedTabsTest {
    private val news = SavedTab("https://news.example.com", "News")
    private val mail = SavedTab("https://mail.example.com/inbox", "")

    @Test
    fun roundTripsSpacesTabsAndTheActiveOnes() {
        val saved = SavedTabs(
            space = "work",
            spaces = listOf(
                SavedSpace("personal", listOf(news, mail), selected = 1),
                SavedSpace("work", emptyList(), selected = null),
            ),
        )
        assertEquals(saved, savedTabs(JSONObject(saved.json().toString())))
    }

    @Test
    fun readsAFileWithNoActiveSpaceOrTab() {
        val json = """{"spaces":[{"id":"personal","tabs":[{"url":"https://news.example.com","title":"News"}]}]}"""
        assertEquals(SavedTabs(null, listOf(SavedSpace("personal", listOf(news), null))), savedTabs(JSONObject(json)))
    }
}
