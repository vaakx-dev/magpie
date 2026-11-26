package app.magpie.browser.settings

import android.content.SharedPreferences
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.core.content.edit
import app.magpie.browser.dark.ForceDark
import app.magpie.browser.focus.Distraction
import app.magpie.browser.focus.Focus
import app.magpie.browser.password.Service
import app.magpie.browser.privacy.Privacy
import app.magpie.browser.site.Site
import app.magpie.browser.sponsorblock.SponsorBlock
import app.magpie.browser.update.Channel
import java.net.URLEncoder
import kotlin.properties.ReadWriteProperty
import kotlin.reflect.KProperty

enum class ThemeMode(val label: String) { System("System"), Light("Light"), Dark("Dark") }

enum class MenuSide(val label: String) {
    Left("Left"),
    Right("Right");

    val placement: String get() = "Menu on the ${label.lowercase()}"
}

enum class SearchEngine(val label: String, private val template: String) {
    DuckDuckGo("DuckDuckGo", "https://duckduckgo.com/?q=%s"),
    Google("Google", "https://www.google.com/search?q=%s"),
    Brave("Brave Search", "https://search.brave.com/search?q=%s"),
    Bing("Bing", "https://www.bing.com/search?q=%s");

    fun search(query: String): String = template.replace("%s", URLEncoder.encode(query, Charsets.UTF_8.name()))
}

private class Stored<T>(initial: T, private val save: (T) -> Unit) : ReadWriteProperty<Any?, T> {
    private var state by mutableStateOf(initial)

    override fun getValue(thisRef: Any?, property: KProperty<*>): T = state

    override fun setValue(thisRef: Any?, property: KProperty<*>, value: T) {
        state = value
        save(value)
    }
}

private const val THEME = "theme"
private const val SEARCH_ENGINE = "search_engine"
private const val AD_BLOCK = "ad_block"
private const val STRIP_TRACKING = "strip_tracking"
private const val TRACKING_PROTECTION = "tracking_protection"
private const val HTTPS_ONLY = "https_only"
private const val GLOBAL_PRIVACY_CONTROL = "global_privacy_control"
private const val CLEAR_ON_EXIT = "clear_on_exit"
private const val PAUSED_SITES = "paused_sites"
private const val FORCE_DARK = "force_dark"
private const val FORCE_DARK_SITES = "force_dark_sites"
private const val MENU_SIDE = "menu_side"
private const val SWIPE_TABS = "swipe_tabs"
private const val BACKGROUND_PLAY = "background_play"
private const val AUTO_UPDATE = "auto_update"
private const val UPDATE_CHANNEL = "update_channel"
private const val PASSWORD_SERVICE = "password_service"
private const val SPONSORBLOCK = "sponsorblock"

class Settings(private val preferences: SharedPreferences) {
    var theme: ThemeMode by stored(enumValue(THEME, ThemeMode.System)) { putString(THEME, it.name) }

    var menuSide: MenuSide by stored(enumValue(MENU_SIDE, MenuSide.Right)) { putString(MENU_SIDE, it.name) }

    var searchEngine: SearchEngine by stored(enumValue(SEARCH_ENGINE, SearchEngine.DuckDuckGo)) {
        putString(SEARCH_ENGINE, it.name)
    }

    var privacy: Privacy by stored(storedPrivacy()) {
        putBoolean(AD_BLOCK, it.adBlock)
        putBoolean(STRIP_TRACKING, it.stripTracking)
        putBoolean(TRACKING_PROTECTION, it.trackingProtection)
        putBoolean(HTTPS_ONLY, it.httpsOnly)
        putBoolean(GLOBAL_PRIVACY_CONTROL, it.globalPrivacyControl)
        putBoolean(CLEAR_ON_EXIT, it.clearOnExit)
        putStringSet(PAUSED_SITES, it.pausedSites.map(Site::name).toSet())
    }

    var forceDark: ForceDark by stored(ForceDark.stored(preferences.getBoolean(FORCE_DARK, false), preferences.getString(FORCE_DARK_SITES, null))) {
        putBoolean(FORCE_DARK, it.enabled)
        putString(FORCE_DARK_SITES, it.encodedSites())
    }

    var swipeTabs: Boolean by stored(preferences.getBoolean(SWIPE_TABS, true)) { putBoolean(SWIPE_TABS, it) }

    var backgroundPlay: Boolean by stored(preferences.getBoolean(BACKGROUND_PLAY, true)) { putBoolean(BACKGROUND_PLAY, it) }

    var autoUpdate: Boolean by stored(preferences.getBoolean(AUTO_UPDATE, true)) { putBoolean(AUTO_UPDATE, it) }

    var updateChannel: Channel by stored(enumValue(UPDATE_CHANNEL, Channel.Stable)) { putString(UPDATE_CHANNEL, it.name) }

    var passwordService: Service by stored(enumValue(PASSWORD_SERVICE, Service.SystemAutofill)) {
        putString(PASSWORD_SERVICE, it.name)
    }

    var focus: Focus by stored(storedFocus()) { focus ->
        Distraction.entries.forEach { putBoolean(it.key, focus.blocks(it)) }
    }

    var sponsorBlock: SponsorBlock by stored(SponsorBlock(preferences.getString(SPONSORBLOCK, null).orEmpty())) {
        putString(SPONSORBLOCK, it.config)
    }

    private fun <T> stored(initial: T, write: SharedPreferences.Editor.(T) -> Unit): ReadWriteProperty<Any?, T> =
        Stored(initial) { value -> preferences.edit { write(value) } }

    private inline fun <reified T : Enum<T>> enumValue(key: String, default: T): T =
        enumValues<T>().find { it.name == preferences.getString(key, null) } ?: default

    private fun storedFocus(): Focus =
        Focus(Distraction.entries.filter { preferences.getBoolean(it.key, it.blockedByDefault) }.toSet())

    private fun storedPrivacy(): Privacy {
        val defaults = Privacy()
        return Privacy(
            adBlock = preferences.getBoolean(AD_BLOCK, defaults.adBlock),
            stripTracking = preferences.getBoolean(STRIP_TRACKING, defaults.stripTracking),
            trackingProtection = preferences.getBoolean(TRACKING_PROTECTION, defaults.trackingProtection),
            httpsOnly = preferences.getBoolean(HTTPS_ONLY, defaults.httpsOnly),
            globalPrivacyControl = preferences.getBoolean(GLOBAL_PRIVACY_CONTROL, defaults.globalPrivacyControl),
            clearOnExit = preferences.getBoolean(CLEAR_ON_EXIT, defaults.clearOnExit),
            pausedSites = preferences.getStringSet(PAUSED_SITES, null).orEmpty().map(::Site).toSet(),
        )
    }
}
