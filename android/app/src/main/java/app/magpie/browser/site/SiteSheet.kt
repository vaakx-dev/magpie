package app.magpie.browser.site

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.DarkMode
import androidx.compose.material.icons.outlined.DeleteSweep
import androidx.compose.material.icons.outlined.LocationOn
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material.icons.outlined.Mic
import androidx.compose.material.icons.outlined.NoEncryption
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material.icons.outlined.PlayCircle
import androidx.compose.material.icons.outlined.RestartAlt
import androidx.compose.material.icons.outlined.Shield
import androidx.compose.material.icons.outlined.Storage
import androidx.compose.material.icons.outlined.Videocam
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.vector.ImageVector
import app.magpie.browser.browser.Browser
import app.magpie.browser.browser.Tab
import app.magpie.browser.favicon.Origin
import app.magpie.browser.permission.Decision
import app.magpie.browser.permission.Kind
import app.magpie.browser.space.Profile
import app.magpie.browser.theme.Confirmation
import app.magpie.browser.theme.Entry
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Section
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.Title
import app.magpie.browser.theme.ToggleEntry

private val KIND_ICONS = mapOf(
    Kind.Camera to Icons.Outlined.Videocam,
    Kind.Microphone to Icons.Outlined.Mic,
    Kind.Location to Icons.Outlined.LocationOn,
    Kind.Notifications to Icons.Outlined.Notifications,
    Kind.Storage to Icons.Outlined.Storage,
    Kind.Autoplay to Icons.Outlined.PlayCircle,
)

private fun connection(tab: Tab): Pair<ImageVector, String> =
    if (tab.secure) Icons.Outlined.Lock to "Connection is secure" else Icons.Outlined.NoEncryption to "Not secure"

@Composable
private fun Connection(tab: Tab, origin: Origin) {
    val (icon, label) = connection(tab)
    Entry(icon, label, onClick = {}, detail = origin.value)
}

@Composable
private fun Shield(browser: Browser, tab: Tab, site: Site) {
    val privacy = browser.settings.privacy
    if (!privacy.adBlock) return Entry(Icons.Outlined.Shield, "Magpie Shield", onClick = {}, detail = "Turned off for every site in Settings", enabled = false)
    val blocking = privacy.blocksAdsOn(site)
    ToggleEntry(
        Icons.Outlined.Shield,
        "Magpie Shield",
        checked = blocking,
        onChange = { browser.toggleShield(tab, site) },
        detail = if (blocking) "Blocking ads on this site" else "Paused on this site",
    )
}

@Composable
private fun ForceDark(browser: Browser, site: Site) {
    val settings = browser.settings
    val forceDark = settings.forceDark
    ToggleEntry(
        Icons.Outlined.DarkMode,
        "Force dark mode",
        checked = forceDark.on(site),
        onChange = { settings.forceDark = forceDark.toggle(site) },
        detail = if (site in forceDark.sites) "Set for this site" else "Follows your setting",
    )
}

@Composable
private fun Permissions(browser: Browser, origin: Origin) {
    val permissions = browser.permissions
    val saved = permissions.site(origin)
    Section("Permissions")
    if (saved.isEmpty()) return Note("This site hasn't been given any permissions.")
    saved.keys.sorted().forEach { kind ->
        val allowed = saved.getValue(kind) == Decision.Allow
        ToggleEntry(
            KIND_ICONS.getValue(kind),
            kind.label,
            checked = allowed,
            onChange = { permissions.remember(origin, setOf(kind), if (it) Decision.Allow else Decision.Block) },
            detail = if (allowed) "Allowed" else "Blocked",
        )
    }
    Entry(Icons.Outlined.RestartAlt, "Reset permissions", onClick = { permissions.forget(origin) }, detail = "Ask again next time")
}

@Composable
private fun ClearData(browser: Browser, site: Site, onDone: () -> Unit) {
    var confirming by remember { mutableStateOf(false) }
    Entry(Icons.Outlined.DeleteSweep, "Clear site data", onClick = { confirming = true }, detail = "Cookies, storage and cache for ${site.name}")
    if (!confirming) return
    Confirmation(
        "Clear data for ${site.name}?",
        "Deletes this site's cookies, storage and cache. You'll be signed out of it.",
        "Clear",
        onConfirm = {
            browser.engine.clear(site)
            onDone()
        },
        onDismiss = { confirming = false },
    )
}

@Composable
fun SiteSheet(browser: Browser, tab: Tab, onDismiss: () -> Unit) {
    val origin = Origin.of(tab.url)
    val site = Site.of(tab.url)
    Sheet(onDismiss) {
        if (origin == null || site == null) {
            Title(tab.label)
            return@Sheet Note("Site settings are only for web pages.")
        }
        Title(origin.address)
        Connection(tab, origin)
        Section("This site")
        Shield(browser, tab, site)
        ForceDark(browser, site)
        if (tab.space.profile != Profile.Private) Permissions(browser, origin)
        Section("Data")
        ClearData(browser, site, onDismiss)
    }
}
