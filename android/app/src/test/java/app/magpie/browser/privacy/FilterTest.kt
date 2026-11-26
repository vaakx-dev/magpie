package app.magpie.browser.privacy

import org.junit.Assert.assertEquals
import org.junit.Test

class FilterTest {
    @Test
    fun passesUrlsThroughWithoutTheCore() {
        val url = "https://example.com/?utm_source=feed"
        assertEquals(url, Filter.clean(url, Privacy()))
    }
}
