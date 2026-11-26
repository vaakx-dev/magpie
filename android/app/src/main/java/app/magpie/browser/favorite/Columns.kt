package app.magpie.browser.favorite

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

val TILE_GAP = 4.dp
private val MIN_TILE = 72.dp
private val COLUMN_RANGE = 2..6

fun columns(width: Dp): Int = ((width + TILE_GAP) / (MIN_TILE + TILE_GAP)).toInt().coerceIn(COLUMN_RANGE)
