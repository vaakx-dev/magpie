package app.magpie.browser.menu

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ChromeReaderMode
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.Bookmarks
import androidx.compose.material.icons.outlined.DarkMode
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.History
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Remove
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material.icons.outlined.ZoomIn
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser
import app.magpie.browser.browser.Panel
import app.magpie.browser.browser.Step
import app.magpie.browser.browser.Tab
import app.magpie.browser.browser.Zoom
import app.magpie.browser.password.Service
import app.magpie.browser.theme.Toggle

private val ZOOM_LEVEL_WIDTH = 48.dp
private val ZOOM_BUTTON_SIZE = 36.dp
private val ZOOM_LEVEL_SHAPE = RoundedCornerShape(6.dp)

class Menu(val browser: Browser, private val onOpen: (Panel) -> Unit, private val onDismiss: () -> Unit) {
    var layer by mutableStateOf(Layer.Root)
        private set

    val page: Tab? get() = browser.tab?.takeUnless { it.blank }

    fun enter(layer: Layer) {
        this.layer = layer
    }

    fun show(panel: Panel) = onOpen(panel)

    fun run(action: () -> Unit) {
        action()
        onDismiss()
    }

    fun load(url: String) = run { browser.load(url) }
}

@Composable
private fun ZoomButton(icon: ImageVector, description: String, onClick: (() -> Unit)?) {
    val colors = MaterialTheme.colorScheme
    IconButton(enabled = onClick != null, onClick = onClick ?: {}, modifier = Modifier.size(ZOOM_BUTTON_SIZE)) {
        Icon(icon, contentDescription = description, Modifier.size(ICON_SIZE), tint = if (onClick != null) colors.onSurface else colors.outline)
    }
}

@Composable
private fun ZoomLevel(zoom: Zoom, onReset: (() -> Unit)?) {
    val colors = MaterialTheme.colorScheme
    Text(
        zoom.toString(),
        color = if (onReset != null) colors.onSurface else colors.outline,
        textAlign = TextAlign.Center,
        style = MaterialTheme.typography.bodyMedium,
        modifier = Modifier
            .width(ZOOM_LEVEL_WIDTH)
            .clip(ZOOM_LEVEL_SHAPE)
            .clickable(enabled = onReset != null, onClick = onReset ?: {})
            .padding(vertical = 8.dp),
    )
}

@Composable
private fun ZoomBand(tab: Tab?) {
    val stepper = { step: Step -> tab?.let { { it.zoom(step) } } }
    Line(leading = { Glyph(Icons.Outlined.ZoomIn) }, label = "Zoom") {
        Row(verticalAlignment = Alignment.CenterVertically) {
            ZoomButton(Icons.Outlined.Remove, "Zoom out", stepper(Step.Out))
            ZoomLevel(tab?.zoom ?: Zoom.DEFAULT, stepper(Step.Reset))
            ZoomButton(Icons.Outlined.Add, "Zoom in", stepper(Step.In))
        }
    }
}

@Composable
private fun RootLayer(menu: Menu) {
    val browser = menu.browser
    val page = menu.page
    val settings = browser.settings
    Item(Icons.Outlined.Add, "New Tab", onClick = { menu.run { browser.open() } })
    Item(Icons.Outlined.VisibilityOff, "New Private Tab", onClick = { menu.run { browser.openPrivate() } })
    Divider()
    Drill(Icons.Outlined.History, "History", onEnter = { menu.enter(Layer.History) })
    Drill(Icons.Outlined.Bookmarks, "Bookmarks", onEnter = { menu.enter(Layer.Bookmarks) })
    Item(Icons.Outlined.Download, "Downloads", onClick = { menu.show(Panel.Downloads) })
    if (settings.passwordService == Service.BuiltIn) Drill(Icons.Outlined.Key, "Passwords", onEnter = { menu.enter(Layer.Passwords) })
    Divider()
    ZoomBand(browser.tab)
    Divider()
    Item(Icons.Outlined.Search, "Find in Page", onClick = page?.let { { menu.show(Panel.Find) } })
    Item(Icons.AutoMirrored.Outlined.ChromeReaderMode, "Reader Mode", onClick = page?.let { { menu.run(it::toggleReader) } })
    Item(Icons.Outlined.DarkMode, "Force Dark Mode", onClick = { settings.forceDark = !settings.forceDark }) {
        Toggle(settings.forceDark, onChange = { settings.forceDark = it })
    }
    Divider()
    Item(Icons.Outlined.Settings, "Settings", onClick = { menu.show(Panel.Settings) })
}

@Composable
fun MenuPopup(browser: Browser, onOpen: (Panel) -> Unit, onDismiss: () -> Unit) {
    val menu = remember(browser) { Menu(browser, onOpen, onDismiss) }
    Floating(browser.settings.menuSide, onDismiss) {
        when (menu.layer) {
            Layer.Root -> RootLayer(menu)
            Layer.History -> HistoryLayer(menu)
            Layer.Bookmarks -> BookmarksLayer(menu)
            Layer.Passwords -> PasswordsLayer(menu)
        }
    }
}
