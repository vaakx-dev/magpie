package app.magpie.browser.address

import app.magpie.browser.settings.SearchEngine
import org.junit.Assert.assertEquals
import org.junit.Test

class AddressTest {
    private fun resolved(input: String) = resolve(input, SearchEngine.DuckDuckGo)

    @Test
    fun keepsAddressesThatHaveAScheme() {
        assertEquals("http://example.com/a?b=1", resolved("  http://example.com/a?b=1 "))
        assertEquals("about:config", resolved("about:config"))
    }

    @Test
    fun addsHttpsToHosts() {
        assertEquals("https://magpie.app", resolved("magpie.app"))
        assertEquals("https://news.example.co.uk/story#top", resolved("news.example.co.uk/story#top"))
        assertEquals("https://localhost:8080/admin", resolved("localhost:8080/admin"))
        assertEquals("https://192.168.1.10", resolved("192.168.1.10"))
    }

    @Test
    fun searchesEverythingElse() {
        assertEquals("https://duckduckgo.com/?q=australian+magpie", resolved("australian magpie"))
        assertEquals("https://duckduckgo.com/?q=2.5", resolved("2.5"))
        assertEquals("https://duckduckgo.com/?q=c%2B%2B+tutorial", resolved("c++ tutorial"))
    }
}
