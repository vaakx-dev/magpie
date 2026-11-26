package app.magpie.browser.favorite

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class EditTest {
    private val favorite = Favorite("https://old.example", "Old")

    @Test
    fun addsHttpsToUrlsWithoutScheme() {
        assertEquals(Favorite("https://example.com", "Example"), favorite.edited("Example", "  example.com "))
    }

    @Test
    fun keepsUrlsThatHaveScheme() {
        assertEquals("http://x.org", favorite.edited("X", "http://x.org")?.url)
    }

    @Test
    fun trimsTitleAndKeepsOldTitleWhenBlank() {
        assertEquals("News", favorite.edited("  News  ", "example.com")?.title)
        assertEquals("Old", favorite.edited("   ", "example.com")?.title)
    }

    @Test
    fun rejectsBlankAndHostlessUrls() {
        assertNull(favorite.edited("Title", "   "))
        assertNull(favorite.edited("Title", "https://"))
        assertNull(favorite.edited("Title", "not a url"))
    }
}
