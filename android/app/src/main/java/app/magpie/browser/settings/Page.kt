package app.magpie.browser.settings

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ChromeReaderMode
import androidx.compose.material.icons.outlined.DarkMode
import androidx.compose.material.icons.outlined.Description
import androidx.compose.material.icons.outlined.Headphones
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.SystemUpdate
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.text.input.KeyboardType
import app.magpie.browser.BuildConfig
import app.magpie.browser.browser.Browser
import app.magpie.browser.core.MagpieCore
import app.magpie.browser.focus.Distraction
import app.magpie.browser.sponsorblock.ENABLED
import app.magpie.browser.sponsorblock.SHOW_SKIP_NOTICE
import app.magpie.browser.sponsorblock.View
import app.magpie.browser.theme.Choices
import app.magpie.browser.theme.Field
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Section
import app.magpie.browser.theme.TextAction
import app.magpie.browser.update.Channel
import app.magpie.browser.update.offer
import app.magpie.browser.update.summary
import app.magpie.browser.update.versionLabel
import org.mozilla.geckoview.BuildConfig as Gecko

@Composable
private fun Appearance(settings: Settings) {
    Section("Theme")
    Choices(ThemeMode.entries, settings.theme, ThemeMode::label) { settings.theme = it }
    Section("Menu side")
    Choices(MenuSide.entries, settings.menuSide, MenuSide::label) { settings.menuSide = it }
}

@Composable
private fun CustomSearch(settings: Settings) {
    val template = rememberTextFieldState(settings.search.custom)
    LaunchedEffect(template) {
        snapshotFlow { template.text.trim().toString() }.collect { settings.search = settings.search.copy(custom = it) }
    }
    Section("Custom search engine")
    Field(template, "Search address", KeyboardType.Uri)
    Note(
        if (settings.search.customValid) {
            "%s is replaced with your search terms"
        } else {
            "Enter an http or https address with %s where the search terms go. Until then Magpie searches DuckDuckGo."
        },
    )
}

@Composable
private fun Search(settings: Settings) {
    Section("Default search engine")
    Card(
        SearchEngine.entries.map { engine ->
            { PickItem(engine.label, settings.search.engine == engine) { settings.search = settings.search.copy(engine = engine) } }
        },
    )
    if (settings.search.engine == SearchEngine.Custom) CustomSearch(settings)
}

private const val NEEDS_CORE = "Needs Magpie's core library, which isn't in this build."

@Composable
private fun SkipSwitches(settings: Settings, view: View) {
    fun switch(key: String, on: Boolean) {
        settings.sponsorBlock = settings.sponsorBlock.switch(key, on)
    }
    Card(
        listOf(
            {
                SwitchItem(
                    null,
                    "SponsorBlock (YouTube)",
                    checked = view.enabled,
                    onChange = { switch(ENABLED, it) },
                    detail = "Skips segments the SponsorBlock community marks in YouTube videos",
                )
            },
            {
                SwitchItem(
                    null,
                    "Show skip notice",
                    checked = view.showSkipNotice,
                    onChange = { switch(SHOW_SKIP_NOTICE, it) },
                    detail = "Says what was skipped, with a button to go back",
                )
            },
        ),
    )
    Section("Skip these segments")
    Card(
        view.categories.map { category ->
            { SwitchItem(null, category.label, checked = category.on, onChange = { switch(category.key, it) }, detail = category.description) }
        },
    )
}

@Composable
private fun PausedChannels(settings: Settings, channels: List<String>) {
    if (channels.isEmpty()) return
    Section("Paused channels")
    Card(
        channels.map { channel ->
            {
                Item(null, channel, onClick = {}, detail = "Nothing is skipped on this channel") {
                    TextAction("Remove", onClick = { settings.sponsorBlock = settings.sponsorBlock.resume(channel) })
                }
            }
        },
    )
}

@Composable
private fun SponsorBlockSettings(settings: Settings) {
    Section("SponsorBlock")
    val view = settings.sponsorBlock.view ?: return Note(NEEDS_CORE)
    SkipSwitches(settings, view)
    PausedChannels(settings, view.channels)
}

@Composable
private fun DistractionBlocker(settings: Settings) {
    Section("Distraction Blocker")
    if (!MagpieCore.loaded) Note(NEEDS_CORE)
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
    SponsorBlockSettings(settings)
    DistractionBlocker(settings)
    Section("Background play")
    Card(
        listOf {
            SwitchItem(
                Icons.Outlined.Headphones,
                "Keep media playing in background",
                checked = settings.backgroundPlay,
                onChange = { settings.backgroundPlay = it },
                detail = "Videos and music keep playing when you switch apps or turn off the screen",
            )
        },
    )
    Section("Force dark mode")
    Card(
        listOf {
            SwitchItem(
                Icons.Outlined.DarkMode,
                "Force dark mode",
                checked = settings.forceDark.enabled,
                onChange = { settings.forceDark = settings.forceDark.copy(enabled = it) },
                detail = "Default for every site. Change it for one site from the menu.",
            )
        },
    )
    Section("Reader mode")
    Card(listOf { Item(Icons.AutoMirrored.Outlined.ChromeReaderMode, "Reader mode", onClick = {}, detail = "Open it from the menu on any article") })
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
    Section("Update channel")
    Choices(Channel.entries, settings.updateChannel, Channel::label) {
        settings.updateChannel = it
        updates.check()
    }
    Note("Nightly builds come from the latest code every 3 hours and may be unstable")
}

@Composable
private fun About(browser: Browser, onClose: () -> Unit) {
    Card(
        listOf(
            { Item(null, "Version", onClick = {}, detail = versionLabel(browser.updates.version, BuildConfig.BUILD_SHA)) },
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
