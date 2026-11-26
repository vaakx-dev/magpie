package app.magpie.browser.browser

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ClosedTabsTest {
    private fun closed(space: String, number: Int) = ClosedTab(space, SavedTab("https://example.com/$number", "Page $number"), number)

    private fun ClosedTabs.drain(): List<String> = generateSequence { pop() }.map { it.tab.url }.toList()

    @Test
    fun undoesTheMostRecentCloseFirstAndStopsAnnouncingIt() {
        val tabs = ClosedTabs()
        val first = closed("personal", 1)
        val second = closed("personal", 2)
        tabs.push(first)
        tabs.push(second)
        assertEquals(second, tabs.announced)
        assertEquals(second, tabs.pop())
        assertNull(tabs.announced)
        assertEquals(first, tabs.pop())
        assertNull(tabs.pop())
    }

    @Test
    fun keepsOnlyTheNewestCloses() {
        val tabs = ClosedTabs()
        (1..MAX_CLOSED_TABS + 5).forEach { tabs.push(closed("personal", it)) }
        val expected = (MAX_CLOSED_TABS + 5 downTo 6).map { "https://example.com/$it" }
        assertEquals(expected, tabs.drain())
    }

    @Test
    fun forgetsClosesFromARemovedSpace() {
        val tabs = ClosedTabs()
        tabs.push(closed("personal", 1))
        tabs.push(closed("work", 2))
        tabs.forget("work")
        assertNull(tabs.announced)
        assertEquals(listOf("https://example.com/1"), tabs.drain())
    }
}
