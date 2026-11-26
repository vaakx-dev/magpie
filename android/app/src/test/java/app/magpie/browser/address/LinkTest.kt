package app.magpie.browser.address

import android.content.Intent
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class LinkTest {
    @Test
    fun opensWebLinksFromViewIntents() {
        assertEquals("https://en.wikipedia.org/wiki/Magpie", link(Intent.ACTION_VIEW, "https://en.wikipedia.org/wiki/Magpie"))
        assertEquals("HTTP://example.com", link(Intent.ACTION_VIEW, "HTTP://example.com"))
    }

    @Test
    fun ignoresOtherActionsAndMissingData() {
        assertNull(link(Intent.ACTION_MAIN, "https://example.com"))
        assertNull(link(Intent.ACTION_VIEW, null))
    }

    @Test
    fun rejectsUnsafeSchemes() {
        assertNull(link(Intent.ACTION_VIEW, "javascript:alert(1)"))
        assertNull(link(Intent.ACTION_VIEW, "file:///data/data/app.magpie.browser/files/secret"))
        assertNull(link(Intent.ACTION_VIEW, "about:config"))
    }
}
