package app.magpie.browser.favorite

import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.magpie.browser.theme.host

private val TILE_SHAPE = RoundedCornerShape(12.dp)
private val ICON_SIZE = 46.dp

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
            Box(Modifier.size(ICON_SIZE).clip(TILE_SHAPE).background(colors.surface), contentAlignment = Alignment.Center) {
                Text(host(favorite.url).firstOrNull()?.uppercase() ?: "·", fontSize = 18.sp, fontWeight = FontWeight.Bold)
            }
            Text(
                favorite.title,
                Modifier.fillMaxWidth().padding(horizontal = 2.dp),
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
    BoxWithConstraints(Modifier.fillMaxWidth().padding(start = 8.dp, end = 8.dp, top = 14.dp, bottom = 6.dp)) {
        val count = columns(maxWidth)
        Column(verticalArrangement = Arrangement.spacedBy(TILE_GAP)) {
            favorites.list.chunked(count).forEach { row ->
                Row(horizontalArrangement = Arrangement.spacedBy(TILE_GAP)) {
                    row.forEach { Tile(it, favorites, onOpen, Modifier.weight(1f)) }
                    repeat(count - row.size) { Spacer(Modifier.weight(1f)) }
                }
            }
        }
    }
}
