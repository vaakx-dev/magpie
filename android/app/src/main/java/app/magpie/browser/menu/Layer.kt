package app.magpie.browser.menu

import android.widget.Toast
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material.icons.outlined.Bookmarks
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.ManageHistory
import androidx.compose.material.icons.outlined.Password
import androidx.compose.material.icons.outlined.Shield
import androidx.compose.material.icons.outlined.Star
import androidx.compose.material.icons.outlined.StarBorder
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import app.magpie.browser.browser.Panel
import app.magpie.browser.clipboard.Content
import app.magpie.browser.clipboard.copy
import app.magpie.browser.password.Lock
import app.magpie.browser.password.Login
import app.magpie.browser.password.Status
import app.magpie.browser.password.label
import kotlinx.coroutines.launch

enum class Layer { Root, History, Bookmarks, Passwords }

private const val RECENT_HISTORY_COUNT = 15

@Composable
fun HistoryLayer(menu: Menu) {
    val recent = menu.browser.history.visits.take(RECENT_HISTORY_COUNT)
    Header("History", onBack = { menu.enter(Layer.Root) })
    if (recent.isNotEmpty()) {
        Heading("Recent History")
        ScrollList { recent.forEach { Link(it.url, it.title, onOpen = { menu.load(it.url) }) } }
        Divider()
    }
    Item(Icons.Outlined.ManageHistory, "Manage History", onClick = { menu.show(Panel.History) })
}

@Composable
private fun BookmarkPage(menu: Menu) {
    val page = menu.page ?: return
    val favorites = menu.browser.favorites
    val saved = favorites.contains(page.url)
    Item(
        if (saved) Icons.Outlined.Star else Icons.Outlined.StarBorder,
        if (saved) "Remove Bookmark" else "Bookmark This Page",
        onClick = { favorites.toggle(page.url, page.title) },
    )
    Divider()
}

@Composable
fun BookmarksLayer(menu: Menu) {
    val favorites = menu.browser.favorites.list
    Header("Bookmarks", onBack = { menu.enter(Layer.Root) })
    BookmarkPage(menu)
    if (favorites.isNotEmpty()) {
        Heading("Favorites")
        ScrollList { favorites.forEach { Link(it.url, it.title, onOpen = { menu.load(it.url) }) } }
        Divider()
    }
    Item(Icons.Outlined.Bookmarks, "Manage Bookmarks", onClick = { menu.show(Panel.Bookmarks) })
}

@Composable
private fun SiteLogins(menu: Menu, logins: List<Login>) {
    if (logins.isEmpty()) return
    val context = LocalContext.current
    Divider()
    Heading("For this site")
    ScrollList {
        logins.forEach { login ->
            val copyPassword = {
                menu.run {
                    copy(context, "Password", login.password, Content.Secret)
                    Toast.makeText(context, "Password copied", Toast.LENGTH_SHORT).show()
                }
            }
            Item(Icons.Outlined.Key, login.username.ifBlank { login.name }, onClick = copyPassword) { Detail(login.name) }
        }
    }
}

@Composable
private fun Generate(menu: Menu) {
    val context = LocalContext.current
    val browser = menu.browser
    val generate = {
        menu.run {
            browser.scope.launch {
                copy(context, "Password", browser.vault.generatePassword(), Content.Secret)
                Toast.makeText(context, "Password copied", Toast.LENGTH_SHORT).show()
            }
        }
    }
    Item(Icons.Outlined.Password, "Generate password", onClick = generate)
}

@Composable
fun PasswordsLayer(menu: Menu) {
    val vault = menu.browser.vault
    val site = menu.page?.url.orEmpty()
    var status by remember { mutableStateOf<Status?>(null) }
    var logins by remember { mutableStateOf(emptyList<Login>()) }
    val unlocked = status?.lock == Lock.Unlocked
    LaunchedEffect(site) {
        val current = vault.status()
        status = current
        if (current.lock == Lock.Unlocked && site.isNotEmpty()) logins = vault.matches(site)
    }
    Header("Passwords", onBack = { menu.enter(Layer.Root) })
    Item(Icons.Outlined.Shield, "Bitwarden", onClick = { menu.show(Panel.Passwords) }) { Detail(label(status)) }
    if (unlocked) SiteLogins(menu, logins)
    Divider()
    if (unlocked) Generate(menu)
    Item(Icons.AutoMirrored.Outlined.OpenInNew, "Open Bitwarden", onClick = { menu.show(Panel.Passwords) })
}
