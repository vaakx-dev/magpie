package app.magpie.browser.favicon

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class OriginTest {
    @Test
    fun keepsOnlySchemeHostAndPortOfWebPages() {
        assertEquals("https://www.youtube.com", Origin.of("https://WWW.YouTube.com/watch?v=1#t")?.value)
        assertEquals("http://localhost:8080", Origin.of("http://localhost:8080/a/b")?.value)
    }

    @Test
    fun rejectsPagesThatHaveNoSiteToAsk() {
        assertNull(Origin.of("about:blank"))
        assertNull(Origin.of("file:///sdcard/page.html"))
        assertNull(Origin.of("not a url"))
    }

    @Test
    fun namesCacheFilesSafelyAndDistinctlyPerOrigin() {
        val plain = Origin.of("https://example.com")!!.fileName
        val ported = Origin.of("https://example.com:8443")!!.fileName
        assertEquals("https___example.com.png", plain)
        assertEquals("https___example.com_8443.png", ported)
    }
}
