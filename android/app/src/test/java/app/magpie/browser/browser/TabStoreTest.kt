package app.magpie.browser.browser

import app.magpie.browser.storage.JsonFile
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

class TabStoreTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val killed = CoroutineScope(Job().apply { cancel() })
    private val open = SavedTabs(
        space = "work",
        spaces = listOf(
            SavedSpace("personal", listOf(SavedTab("https://news.example.com", "News")), selected = 0),
            SavedSpace("work", listOf(SavedTab("https://mail.example.com", "Mail")), selected = 0),
        ),
    )
    private val empty = SavedTabs(space = null, spaces = listOf(SavedSpace("personal", emptyList(), null)))

    private fun store(): TabStore = TabStore(JsonFile(File(folder.root, "tabs.json"), killed))

    @Test
    fun openTabsSurviveAProcessKillAfterFlush() {
        val before = store()
        before.restore()
        before.flush(open)
        assertEquals(open, store().restore())
    }

    @Test
    fun startupNeverOverwritesSavedTabsBeforeRestoring() {
        store().apply { restore() }.flush(open)
        val starting = store()
        starting.flush(empty)
        assertEquals(open, starting.restore())
    }

    @Test
    fun aFreshInstallStartsWithNoTabs() {
        assertNull(store().restore())
    }
}
