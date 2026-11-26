package app.magpie.browser.favorite

import androidx.compose.ui.unit.dp
import org.junit.Assert.assertEquals
import org.junit.Test

class ColumnsTest {
    @Test
    fun fitsAsManyMinimumTilesAsTheWidthAllowsWithinBounds() {
        assertEquals(2, columns(100.dp))
        assertEquals(4, columns(344.dp))
        assertEquals(5, columns(400.dp))
        assertEquals(6, columns(1200.dp))
    }
}
