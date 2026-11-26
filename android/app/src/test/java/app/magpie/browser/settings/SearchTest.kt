package app.magpie.browser.settings

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SearchTest {
    private fun custom(template: String) = Search(SearchEngine.Custom, template)

    @Test
    fun customTemplatesGetTheEncodedQueryEverywhereItAppears() {
        val search = custom("https://search.example.org/find?q=%s&echo=%s")
        assertTrue(search.customValid)
        assertEquals("https://search.example.org/find?q=c%2B%2B+%26+rust&echo=c%2B%2B+%26+rust", search.url("c++ & rust"))
    }

    @Test
    fun invalidCustomTemplatesFallBackToDuckDuckGo() {
        listOf("https://search.example.org/find", "javascript:alert(%s)", "ftp://files.example.org/%s", "not a url %s").forEach { template ->
            val search = custom(template)
            assertFalse(template, search.customValid)
            assertEquals(template, "https://duckduckgo.com/?q=magpie", search.url("magpie"))
        }
    }

    @Test
    fun builtInEnginesIgnoreTheCustomTemplate() {
        assertEquals("https://kagi.com/search?q=magpie+nest", Search(SearchEngine.Kagi, "https://search.example.org/?q=%s").url("magpie nest"))
    }
}
