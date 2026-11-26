package app.magpie.browser.settings

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ChromeReaderMode
import androidx.compose.material.icons.outlined.DarkMode
import androidx.compose.material.icons.outlined.Description
import androidx.compose.material.icons.outlined.Headphones
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.SystemUpdate
import androidx.compose.runtime.Composable
import app.magpie.browser.browser.Browser
import app.magpie.browser.core.MagpieCore
import app.magpie.browser.focus.Distraction
import app.magpie.browser.theme.Choices
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Section
import app.magpie.browser.update.offer
import app.magpie.browser.update.summary
import org.mozilla.geckoview.BuildConfig as Gecko

@Composable
private fun Appearance(settings: Settings) {
    Section("Theme")
    Choices(ThemeMode.entries, settings.theme, ThemeMode::label) { settings.theme = it }
    Section("Menu side")
    Choices(MenuSide.entries, settings.menuSide, MenuSide::label) { settings.menuSide = it }
}

@Composable
private fun Search(settings: Settings) {
    Section("Default search engine")
    Card(SearchEngine.entries.map { engine -> { PickItem(engine.label, settings.searchEngine == engine) { settings.searchEngine = engine } } })
}

@Composable
private fun DistractionBlocker(settings: Settings) {
    Section("Distraction Blocker")
    if (!MagpieCore.loaded) Note("Needs Magpie's core library, which isn't in this build.")
    Card(
        Distraction.entries.map { distraction ->
            {
                val focus = settings.focus
                SwitchItem(
                    null,
                    distraction.label,
                    checked = focus.blocks(distraction),
                    onChange = { blocked -> settings.focus = if (blocked) focus.block(distraction) else focus.allow(distraction) },
                    detail = distraction.hint,
                )
            }
        },
    )
}

@Composable
private fun Addons(settings: Settings) {
    DistractionBlocker(settings)
    Section("Media")
    Card(
        listOf {
            SwitchItem(
                Icons.Outlined.Headphones,
                "Background play",
                checked = settings.backgroundPlay,
                onChange = { settings.backgroundPlay = it },
                detail = "Keeps videos and audio playing when you switch apps or turn off the screen",
            )
        },
    )
    Section("Page tools")
    Card(
        listOf(
            { Item(Icons.AutoMirrored.Outlined.ChromeReaderMode, "Reader mode", onClick = {}, detail = "Open it from the menu on any article") },
            {
                SwitchItem(
                    Icons.Outlined.DarkMode,
                    "Force dark mode",
                    checked = settings.forceDark,
                    onChange = { settings.forceDark = it },
                    detail = "Darkens pages that have no dark theme",
                )
            },
        ),
    )
}

@Composable
private fun Updates(browser: Browser, onClose: () -> Unit) {
    val updates = browser.updates
    val settings = browser.settings
    val offer = updates.phase.offer
    val installable = updates.installable
    Section("Updates")
    Card(
        listOfNotNull(
            { Item(Icons.Outlined.Refresh, "Check for updates", onClick = updates::check, detail = updates.phase.summary()) },
            installable?.download?.let { download ->
                { Item(Icons.Outlined.SystemUpdate, "Download & Install", onClick = updates::install, detail = download.name) }
            },
            offer?.let {
                {
                    Link(Icons.Outlined.Description, "Release notes", detail = it.notes.ifBlank { null }, onClick = {
                        browser.open().load(it.page)
                        onClose()
                    })
                }
            },
            {
                SwitchItem(
                    null,
                    "Check for updates automatically",
                    checked = settings.autoUpdate,
                    onChange = { settings.autoUpdate = it },
                    detail = "Looks for a new release on GitHub at most every 12 hours",
                )
            },
        ),
    )
}

@Composable
private fun About(browser: Browser, onClose: () -> Unit) {
    Card(
        listOf(
            { Item(null, "Version", onClick = {}, detail = "Magpie ${browser.updates.version}") },
            { Item(null, "GeckoView", onClick = {}, detail = Gecko.MOZILLA_VERSION) },
            { Item(null, "Magpie core", onClick = {}, detail = if (MagpieCore.loaded) "Loaded" else "Not in this build") },
        ),
    )
    Updates(browser, onClose)
}

@Composable
fun Page(category: Category, browser: Browser, onPasswords: () -> Unit, onClose: () -> Unit) {
    when (category) {
        Category.General -> General(browser)
        Category.Appearance -> Appearance(browser.settings)
        Category.Search -> Search(browser.settings)
        Category.Privacy -> Privacy(browser)
        Category.Passwords -> Passwords(browser, onPasswords)
        Category.Addons -> Addons(browser.settings)
        Category.About -> About(browser, onClose)
    }
}
