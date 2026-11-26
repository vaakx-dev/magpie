package app.magpie.browser.settings

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Block
import androidx.compose.material.icons.outlined.DeleteSweep
import androidx.compose.material.icons.outlined.Link
import androidx.compose.material.icons.outlined.Shield
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import app.magpie.browser.browser.Browser
import app.magpie.browser.core.MagpieCore
import app.magpie.browser.theme.Note

@Composable
fun Privacy(browser: Browser) {
    val settings = browser.settings
    val privacy = settings.privacy
    var clearing by remember { mutableStateOf(false) }
    if (!MagpieCore.loaded) Note("Magpie's core library isn't in this build, so the ad blocker and link cleaning are off.")
    Card(
        listOf(
            {
                SwitchItem(
                    Icons.Outlined.Block,
                    "Ad blocker",
                    checked = privacy.adBlock,
                    onChange = { settings.privacy = privacy.copy(adBlock = it) },
                    detail = "Blocks ads and hides their empty boxes",
                )
            },
            {
                SwitchItem(
                    Icons.Outlined.Link,
                    "Strip tracking from URLs",
                    checked = privacy.stripTracking,
                    onChange = { settings.privacy = privacy.copy(stripTracking = it) },
                    detail = "Removes utm_ and click IDs from links",
                )
            },
            {
                SwitchItem(
                    Icons.Outlined.Shield,
                    "Tracking protection",
                    checked = privacy.trackingProtection,
                    onChange = { settings.privacy = privacy.copy(trackingProtection = it) },
                    detail = "Strict: blocks trackers, fingerprinters and cryptominers",
                )
            },
        ),
    )
    Card(listOf { Link(Icons.Outlined.DeleteSweep, "Clear browsing data", { clearing = true }, "History, cookies, storage and cache") })
    if (clearing) ConfirmClearBrowsingData(browser, onDismiss = { clearing = false })
}

@Composable
fun ConfirmClearBrowsingData(browser: Browser, onDismiss: () -> Unit) {
    ConfirmClear(
        "Clear browsing data?",
        "Deletes your history and every profile's cookies, storage and cache. You'll be signed out of every site.",
        onConfirm = {
            browser.clearBrowsingData()
            onDismiss()
        },
        onDismiss = onDismiss,
    )
}
