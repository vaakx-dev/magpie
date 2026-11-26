package app.magpie.browser.download

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test

class RecordTest {
    @Test
    fun roundTripsFinishedAndFailedDownloads() {
        val records = listOf(
            Record("report.pdf", "application/pdf", 2048, Progress.Done, 2048, "content://media/external/downloads/7"),
            Record("stream.bin", "application/octet-stream", null, Progress.Failed, 512, null),
        )
        assertEquals(records, records(JSONObject(records.json().toString())))
    }
}
