package app.magpie.browser.favorite

import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.magpie.browser.favicon.Favicon

private val TILE_SHAPE = RoundedCornerShape(12.dp)
private val ICON_SIZE = 48.dp
private val ROW_GAP = 4.dp

@Composable
private fun Tile(favorite: Favorite, favorites: Favorites, onOpen: (Favorite) -> Unit, modifier: Modifier) {
    val colors = MaterialTheme.colorScheme
    var menu by remember { mutableStateOf(false) }
    Box(modifier) {
        Column(
            Modifier.fillMaxWidth().clip(TILE_SHAPE).combinedClickable(onLongClick = { menu = true }) { onOpen(favorite) }.padding(vertical = 4.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Favicon(favorite.url, ICON_SIZE)
            Text(
                favorite.title,
                Modifier.fillMaxWidth(),
                fontSize = 11.5.sp,
                color = colors.onSurfaceVariant,
                textAlign = TextAlign.Center,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        FavoriteMenu(favorite, favorites, menu, onDismiss = { menu = false })
    }
}

@Composable
fun Grid(favorites: Favorites, onOpen: (Favorite) -> Unit) {
    BoxWithConstraints(
        Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 14.dp, bottom = 6.dp),
        contentAlignment = Alignment.TopCenter,
    ) {
        val count = columns(maxWidth, favorites.list.size)
        Column(Modifier.width(gridWidth(count)), verticalArrangement = Arrangement.spacedBy(ROW_GAP)) {
            favorites.list.chunked(count).forEach { row ->
                Row(horizontalArrangement = Arrangement.spacedBy(TILE_GAP)) {
                    row.forEach { Tile(it, favorites, onOpen, Modifier.width(TILE_WIDTH)) }
                }
            }
        }
    }
}
