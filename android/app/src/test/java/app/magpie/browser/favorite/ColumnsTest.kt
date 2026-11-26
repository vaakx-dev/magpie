package app.magpie.browser.favorite

import androidx.compose.ui.unit.dp
import org.junit.Assert.assertEquals
import org.junit.Test

class ColumnsTest {
    @Test
    fun fewFavoritesSitTogetherInsteadOfSpreadingAcrossTheScreen() {
        val columns = columns(360.dp, tiles = 2)
        assertEquals(2, columns)
        assertEquals(138.dp, gridWidth(columns))
    }

    @Test
    fun manyFavoritesWrapAtTheColumnsThatFitWithinBounds() {
        assertEquals(1, columns(40.dp, tiles = 3))
        assertEquals(4, columns(359.dp, tiles = 12))
        assertEquals(5, columns(360.dp, tiles = 12))
        assertEquals(6, columns(1200.dp, tiles = 12))
    }
}
