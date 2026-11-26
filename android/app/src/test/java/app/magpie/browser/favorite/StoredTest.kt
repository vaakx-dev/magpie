package app.magpie.browser.favorite

import org.junit.Assert.assertEquals
import org.junit.Test

class StoredTest {
    @Test
    fun seedsYouTubeAndMagpieWhenNothingWasEverSaved() {
        val seeded = listOf(
            Favorite("https://www.youtube.com", "YouTube"),
            Favorite("https://github.com/vaakx-dev/magpie", "Magpie"),
        )
        assertEquals(seeded, stored(null))
    }

    @Test
    fun keepsSavedFavoritesEvenWhenEmpty() {
        assertEquals(emptyList<Favorite>(), stored("[]"))
        assertEquals(listOf(Favorite("https://example.com", "Example")), stored("""[{"url":"https://example.com","title":"Example"}]"""))
    }
}
