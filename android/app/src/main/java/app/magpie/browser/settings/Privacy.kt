package app.magpie.browser.settings

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Block
import androidx.compose.material.icons.outlined.DeleteSweep
import androidx.compose.material.icons.outlined.Https
import androidx.compose.material.icons.outlined.Link
import androidx.compose.material.icons.outlined.PanTool
import androidx.compose.material.icons.outlined.Shield
import androidx.compose.material.icons.outlined.Tune
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import app.magpie.browser.browser.Browser
import app.magpie.browser.core.MagpieCore
import app.magpie.browser.permission.Permissions
import app.magpie.browser.permission.summary
import app.magpie.browser.site.Site
import app.magpie.browser.theme.Confirmation
import app.magpie.browser.theme.Entry
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Section
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.TextAction
import app.magpie.browser.theme.Title

private fun count(sites: Int): String = if (sites == 1) "1 site" else "$sites sites"

@Composable
private fun Shield(settings: Settings) {
    val privacy = settings.privacy
    Section("Magpie Shield")
    Card(
        listOf {
            SwitchItem(
                Icons.Outlined.Block,
                "Ad & tracker blocker",
                checked = privacy.adBlock,
                onChange = { settings.privacy = privacy.copy(adBlock = it) },
                detail = "Magpie Shield blocks ads and trackers on every site",
            )
        },
    )
}

@Composable
private fun Tracking(settings: Settings) {
    val privacy = settings.privacy
    Section("Tracking")
    Card(
        listOf(
            {
                SwitchItem(
                    Icons.Outlined.Shield,
                    "Tracking protection",
                    checked = privacy.trackingProtection,
                    onChange = { settings.privacy = privacy.copy(trackingProtection = it) },
                    detail = "Strict: blocks trackers, fingerprinters and cryptominers",
                )
            },
            {
                SwitchItem(
                    Icons.Outlined.Https,
                    "HTTPS-Only mode",
                    checked = privacy.httpsOnly,
                    onChange = { settings.privacy = privacy.copy(httpsOnly = it) },
                    detail = "Opens http:// sites over https:// and warns before any that can't",
                )
            },
            {
                SwitchItem(
                    Icons.Outlined.PanTool,
                    "Ask sites not to track you",
                    checked = privacy.globalPrivacyControl,
                    onChange = { settings.privacy = privacy.copy(globalPrivacyControl = it) },
                    detail = "Sends Global Privacy Control, telling sites not to sell or share your data",
                )
            },
            {
                SwitchItem(
                    Icons.Outlined.Link,
                    "Strip tracking from links",
                    checked = privacy.stripTracking,
                    onChange = { settings.privacy = privacy.copy(stripTracking = it) },
                    detail = "Removes utm_ and click IDs from links",
                )
            },
        ),
    )
}

@Composable
private fun PausedSites(settings: Settings) {
    val paused = settings.privacy.pausedSites.sortedBy(Site::name)
    if (paused.isEmpty()) return
    Section("Magpie Shield paused on")
    Card(
        paused.map { site ->
            {
                Item(null, site.name, onClick = {}) {
                    TextAction("Resume", onClick = { settings.privacy = settings.privacy.resume(site) })
                }
            }
        },
    )
}

@Composable
private fun SitePermissionsSheet(permissions: Permissions, onDismiss: () -> Unit) {
    Sheet(onDismiss) {
        Title("Site permissions")
        val origins = permissions.origins
        if (origins.isEmpty()) Note("Permissions you remember for a site show up here.")
        origins.forEach { origin ->
            Entry(null, origin.address, onClick = {}, detail = summary(permissions.site(origin))) {
                TextAction("Reset", onClick = { permissions.forget(origin) })
            }
        }
    }
}

@Composable
private fun SitePermissions(browser: Browser) {
    var open by remember { mutableStateOf(false) }
    Section("Site permissions")
    Card(listOf { Link(Icons.Outlined.Tune, "Site permissions", { open = true }, count(browser.permissions.origins.size)) })
    if (open) SitePermissionsSheet(browser.permissions, onDismiss = { open = false })
}

@Composable
private fun BrowsingData(browser: Browser) {
    val settings = browser.settings
    val privacy = settings.privacy
    var clearing by remember { mutableStateOf(false) }
    Section("Browsing data")
    Card(
        listOf(
            { Link(Icons.Outlined.DeleteSweep, "Clear browsing data", { clearing = true }, "History, cookies, storage, cache and site permissions") },
            {
                SwitchItem(
                    null,
                    "Clear browsing data on exit",
                    checked = privacy.clearOnExit,
                    onChange = { settings.privacy = privacy.copy(clearOnExit = it) },
                    detail = "Deletes it each time Magpie starts again. Open tabs are kept.",
                )
            },
        ),
    )
    if (clearing) ConfirmClearBrowsingData(browser, onDismiss = { clearing = false })
}

@Composable
fun Privacy(browser: Browser) {
    if (!MagpieCore.loaded) Note("Magpie's core library isn't in this build, so Magpie Shield and link cleaning are off.")
    Shield(browser.settings)
    PausedSites(browser.settings)
    Tracking(browser.settings)
    SitePermissions(browser)
    BrowsingData(browser)
}

@Composable
fun ConfirmClearBrowsingData(browser: Browser, onDismiss: () -> Unit) {
    Confirmation(
        "Clear browsing data?",
        "Deletes your history, saved site permissions and every profile's cookies, storage and cache. You'll be signed out of every site.",
        "Clear",
        onConfirm = {
            browser.clearBrowsingData()
            onDismiss()
        },
        onDismiss = onDismiss,
    )
}
