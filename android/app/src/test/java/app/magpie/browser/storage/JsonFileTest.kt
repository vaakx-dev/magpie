package app.magpie.browser.storage

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

class JsonFileTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val killed = CoroutineScope(Job().apply { cancel() })

    private fun file(): File = File(folder.root, "state.json")

    private fun note(text: String): JSONObject = JSONObject().put("note", text)

    private fun JsonFile.note(): String? = read { it.getString("note") }

    @Test
    fun flushWritesTheLatestStateBeforeTheProcessDies() {
        val before = JsonFile(file(), killed)
        before.write(note("first"))
        before.write(note("second"))
        assertTrue(before.flush())
        assertEquals("second", JsonFile(file(), killed).note())
    }

    @Test
    fun aWriteThatNeverFlushedIsLostWhenTheProcessDies() {
        JsonFile(file(), killed).write(note("unsaved"))
        assertNull(JsonFile(file(), killed).note())
    }

    @Test
    fun aLaterFlushKeepsTheEarlierSavedState() {
        val store = JsonFile(file(), killed)
        store.write(note("saved"))
        store.flush()
        assertTrue(store.flush())
        assertEquals("saved", JsonFile(file(), killed).note())
    }

    @Test
    fun aCorruptFileIsSetAsideInsteadOfOverwritten() {
        file().writeText("{\"note\": ")
        assertNull(JsonFile(file(), killed).note())
        assertFalse(file().exists())
        assertEquals("{\"note\": ", File(folder.root, "state.json.corrupt").readText())
    }
}
