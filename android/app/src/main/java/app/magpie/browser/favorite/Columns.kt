package app.magpie.browser.favorite

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

val TILE_WIDTH = 64.dp
val TILE_GAP = 10.dp
private const val MAX_COLUMNS = 6

fun columns(width: Dp, tiles: Int): Int {
    val fitting = ((width + TILE_GAP) / (TILE_WIDTH + TILE_GAP)).toInt()
    return minOf(fitting, tiles, MAX_COLUMNS).coerceAtLeast(1)
}

fun gridWidth(columns: Int): Dp = TILE_WIDTH * columns + TILE_GAP * (columns - 1)
