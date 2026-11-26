package app.magpie.browser.context

import android.content.Context
import android.content.Intent
import android.widget.Toast
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.Image
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material.icons.outlined.Tab
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser
import app.magpie.browser.browser.Tab
import app.magpie.browser.clipboard.Content
import app.magpie.browser.clipboard.copy
import app.magpie.browser.menu.Divider
import app.magpie.browser.menu.Item
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import app.magpie.browser.theme.Sheet
import org.mozilla.geckoview.GeckoSession.ContentDelegate.ContextElement
import org.mozilla.geckoview.GeckoWebExecutor
import org.mozilla.geckoview.WebRequest

private val LIST_PADDING = 10.dp

fun ContextElement.image(): String? = srcUri?.takeIf { type == ContextElement.TYPE_IMAGE }

private class Actions(private val browser: Browser, private val tab: Tab, private val context: Context, private val close: () -> Unit) {
    val privately = tab.space.profile == Profile.Private

    fun openInNewTab(url: String) = openInBackground(url, tab.space, "Opened in new tab")

    fun openInPrivateTab(url: String) = openInBackground(url, Space.PRIVATE, "Opened in Private tab")

    fun copyLink(url: String) = finish("Link copied") { copy(context, "Link", url, Content.Plain) }

    fun share(url: String) = finish {
        val send = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, url)
        context.startActivity(Intent.createChooser(send, null))
    }

    fun save(url: String) = finish {
        val flags = if (privately) GeckoWebExecutor.FETCH_FLAGS_PRIVATE else GeckoWebExecutor.FETCH_FLAGS_NONE
        GeckoWebExecutor(browser.engine.runtime).fetch(WebRequest.Builder(url).referrer(tab.url).build(), flags)
            .accept({ response -> response?.let(browser.downloads::save) }, { notify("Couldn't save image") })
    }

    private fun openInBackground(url: String, space: Space, notice: String) = finish(notice) {
        browser.open(space).load(browser.clean(url))
        browser.select(tab)
    }

    private fun notify(text: String) = Toast.makeText(context, text, Toast.LENGTH_SHORT).show()

    private fun finish(notice: String? = null, action: () -> Unit) {
        close()
        action()
        notice?.let(::notify)
    }
}

@Composable
private fun Header(text: String) {
    Text(
        text,
        modifier = Modifier.padding(horizontal = 10.dp, vertical = 8.dp),
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun LinkActions(actions: Actions, url: String) {
    Item(Icons.Outlined.Tab, "Open in new tab", onClick = { actions.openInNewTab(url) })
    if (!actions.privately) Item(Icons.Outlined.VisibilityOff, "Open in Private tab", onClick = { actions.openInPrivateTab(url) })
    Item(Icons.Outlined.ContentCopy, "Copy link", onClick = { actions.copyLink(url) })
    Item(Icons.Outlined.Share, "Share link", onClick = { actions.share(url) })
}

@Composable
private fun ImageActions(actions: Actions, url: String) {
    Item(Icons.Outlined.Image, "Open image in new tab", onClick = { actions.openInNewTab(url) })
    Item(Icons.Outlined.Download, "Save image", onClick = { actions.save(url) })
    Item(Icons.Outlined.ContentCopy, "Copy image link", onClick = { actions.copyLink(url) })
}

@Composable
fun ContextMenu(browser: Browser, tab: Tab, element: ContextElement) {
    val context = LocalContext.current
    val actions = Actions(browser, tab, context, tab::clearHeld)
    val link = element.linkUri
    val image = element.image()
    Sheet(onDismiss = tab::clearHeld) {
        Column(Modifier.padding(horizontal = LIST_PADDING)) {
            Header(link ?: image.orEmpty())
            link?.let { LinkActions(actions, it) }
            if (link != null && image != null) Divider()
            image?.let { ImageActions(actions, it) }
        }
    }
}
