package app.magpie.browser.favorite

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material.icons.outlined.Star
import androidx.compose.material.icons.outlined.StarBorder
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser
import app.magpie.browser.theme.Entry
import app.magpie.browser.theme.IconAction
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.SiteEntry
import app.magpie.browser.theme.Title

@Composable
private fun Overflow(favorite: Favorite, favorites: Favorites) {
    var menu by remember { mutableStateOf(false) }
    Box {
        IconAction(Icons.Outlined.MoreVert, "Bookmark options", onClick = { menu = true })
        FavoriteMenu(favorite, favorites, menu, onDismiss = { menu = false })
    }
}

@Composable
fun BookmarksSheet(browser: Browser, onDismiss: () -> Unit) {
    val favorites = browser.favorites
    val page = browser.tab?.takeUnless { it.blank }
    Sheet(onDismiss) {
        Title("Bookmarks")
        page?.let {
            val saved = favorites.contains(it.url)
            Entry(
                if (saved) Icons.Outlined.Star else Icons.Outlined.StarBorder,
                if (saved) "Remove this page" else "Bookmark this page",
                detail = it.label,
                onClick = { favorites.toggle(it.url, it.title) },
            )
        }
        if (favorites.list.isEmpty()) Note("Bookmarks show up here and in Favorites on the tabs sheet.")
        LazyColumn(Modifier.heightIn(max = 480.dp)) {
            items(favorites.list, key = { it.url }) { favorite ->
                SiteEntry(
                    favorite.url,
                    favorite.title,
                    onClick = {
                        browser.load(favorite.url)
                        onDismiss()
                    },
                    trailing = { Overflow(favorite, favorites) },
                )
            }
        }
    }
}
