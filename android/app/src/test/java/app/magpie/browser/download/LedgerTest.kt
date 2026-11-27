package app.magpie.browser.download

import app.magpie.browser.storage.JsonFile
import app.magpie.browser.storage.Retention
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

class LedgerTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val killed = CoroutineScope(Job().apply { cancel() })

    private fun ledger(): Ledger = Ledger(JsonFile(File(folder.root, "downloads.json"), killed))

    @Test
    fun onlyFinishedDownloadsFromRegularTabsAreThereAfterARestart() {
        val ledger = ledger()
        val report = Download("report.pdf", "application/pdf", 2048, Retention.Disk)
        val secret = Download("secret.pdf", "application/pdf", 1024, Retention.Memory)
        val running = Download("movie.mp4", "video/mp4", null, Retention.Disk)
        listOf(report, secret, running).forEach(ledger::add)
        ledger.finish(report, Progress.Done)
        ledger.finish(secret, Progress.Done)
        ledger.flush()

        assertEquals(listOf("secret.pdf", "report.pdf"), ledger.list.filter { it.progress == Progress.Done }.map(Download::name))
        assertEquals(listOf("report.pdf"), ledger().list.map(Download::name))
    }
}
