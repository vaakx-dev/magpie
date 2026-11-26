package app.magpie.browser.history

import androidx.core.content.edit
import app.magpie.browser.settings.MemoryPreferences
import app.magpie.browser.storage.JsonFile
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import org.json.JSONArray
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File
import java.time.LocalDateTime
import java.time.ZoneOffset

class HistoryTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val killed = CoroutineScope(Job().apply { cancel() })
    private val preferences = MemoryPreferences()

    private fun path(): File = File(folder.root, "history.json")

    private fun history(): History = History(JsonFile(path(), killed), preferences)

    private fun at(text: String): Long = LocalDateTime.parse(text).toInstant(ZoneOffset.UTC).toEpochMilli()

    @Test
    fun visitsSurviveARestart() {
        history().apply {
            record("https://news.example.com")
            title("https://news.example.com", "News")
            record("https://mail.example.com")
            flush()
        }
        val reopened = history().visits.map { it.url to it.title }
        assertEquals(listOf("https://mail.example.com" to "", "https://news.example.com" to "News"), reopened)
    }

    @Test
    fun savedPreferencesHistoryMovesIntoTheFileOnce() {
        val saved = """[{"url":"https://new.example.com","title":"New"},{"url":"https://old.example.com","title":""}]"""
        preferences.edit { putString("history", saved) }

        val migrated = history().visits.toList()

        val expected = listOf(Visit("https://new.example.com", "New", 0), Visit("https://old.example.com", "", 0))
        assertEquals(expected, migrated)
        assertFalse(preferences.contains("history"))
        assertTrue(path().exists())
        assertEquals(expected, history().visits.toList())
    }

    @Test
    fun unreadablePreferencesHistoryStartsEmptyInsteadOfCrashing() {
        preferences.edit { putString("history", "[{\"url\":") }
        assertTrue(history().visits.isEmpty())
        assertFalse(preferences.contains("history"))
    }

    @Test
    fun aCorruptHistoryFileIsSetAsideAndNotOverwrittenByTheMigration() {
        path().writeText("{\"visits\": [")
        assertTrue(history().visits.isEmpty())
        assertEquals("{\"visits\": [", File(folder.root, "history.json.corrupt").readText())
    }

    @Test
    fun keepsTheNewestFiveThousandVisits() {
        val saved = (1..MAX_VISITS + 1).map { JSONObject().put("url", "https://example.com/$it") }
        preferences.edit { putString("history", JSONArray(saved).toString()) }
        val history = history()
        assertEquals(MAX_VISITS, history.visits.size)

        history.record("https://example.com/newest")

        assertEquals(MAX_VISITS, history.visits.size)
        assertEquals("https://example.com/newest", history.visits.first().url)
        assertEquals("https://example.com/${MAX_VISITS - 1}", history.visits.last().url)
    }

    @Test
    fun forgettingOneVisitKeepsTheRest() {
        val history = history().apply {
            record("https://bank.example.com")
            record("https://news.example.com")
            forget("https://bank.example.com")
            flush()
        }
        assertEquals(listOf("https://news.example.com"), history.visits.map { it.url })
        assertEquals(listOf("https://news.example.com"), history().visits.map { it.url })
    }

    @Test
    fun searchMatchesTitlesAndAddressesIgnoringCase() {
        val visits = listOf(
            Visit("https://news.example.com/", "Morning News", 0),
            Visit("https://magpies.example.org/", "Birds", 0),
            Visit("https://example.com/crows", "", 0),
        )
        assertEquals(listOf(visits[0]), matching(visits, " morning "))
        assertEquals(listOf(visits[1]), matching(visits, "MAGPIES"))
        assertEquals(visits, matching(visits, ""))
        assertTrue(matching(visits, "nothing").isEmpty())
    }

    @Test
    fun visitsAreGroupedByLocalDay() {
        val now = at("2026-11-27T09:00:00")
        val today = Visit("https://a.example/", "", at("2026-11-27T00:30:00"))
        val yesterday = Visit("https://b.example/", "", at("2026-11-26T23:59:00"))
        val lastWeek = Visit("https://c.example/", "", at("2026-11-20T12:00:00"))
        val migrated = Visit("https://d.example/", "", 0)

        val groups = grouped(listOf(today, yesterday, lastWeek, migrated), now, ZoneOffset.UTC)

        val expected = listOf(
            Group(Day.Today, listOf(today)),
            Group(Day.Yesterday, listOf(yesterday)),
            Group(Day.Earlier, listOf(lastWeek, migrated)),
        )
        assertEquals(expected, groups)
    }
}
