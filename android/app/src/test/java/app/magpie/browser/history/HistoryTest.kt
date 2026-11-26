package app.magpie.browser.history

import app.magpie.browser.settings.MemoryPreferences
import androidx.core.content.edit
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class HistoryTest {
    @Test
    fun visitsSurviveARestart() {
        val preferences = MemoryPreferences()
        History(preferences).apply {
            record("https://news.example.com")
            title("https://news.example.com", "News")
            record("https://mail.example.com")
        }
        val expected = listOf(Visit("https://mail.example.com", ""), Visit("https://news.example.com", "News"))
        assertEquals(expected, History(preferences).visits.toList())
    }

    @Test
    fun unreadableHistoryStartsEmptyInsteadOfCrashing() {
        val preferences = MemoryPreferences().apply { edit { putString("history", "[{\"url\":") } }
        assertTrue(History(preferences).visits.isEmpty())
    }
}
