package app.magpie.browser.browser

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import app.magpie.browser.address.resolve
import app.magpie.browser.download.Downloads
import app.magpie.browser.favorite.Favorites
import app.magpie.browser.history.History
import app.magpie.browser.media.Playback
import app.magpie.browser.password.Vault
import app.magpie.browser.privacy.Filter
import app.magpie.browser.settings.Settings
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import app.magpie.browser.space.Spaces
import app.magpie.browser.update.Updates
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch
import org.json.JSONObject
import org.mozilla.geckoview.WebExtension

private const val PREFERENCES = "magpie"

class Browser(private val context: Context) {
    val scope = MainScope()
    private val preferences = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
    val settings = Settings(preferences)
    val spaces = Spaces(preferences)
    val favorites = Favorites(preferences)
    val history = History(preferences)
    val downloads = Downloads(context, scope)
    val vault = Vault()
    val engine = Engine(context, settings, scope)
    val updates = Updates(context, scope, settings)

    val tabs = mutableStateListOf<Tab>()
    var space by mutableStateOf(spaces.saved.first())
        private set
    private val selected = mutableStateMapOf<String, Tab>()
    private var extension: WebExtension? = null
    private var nextTabId = 0L

    val tab: Tab? get() = selected[space.id]

    val spaceTabs: List<Tab> get() = tabs.filter { it.space == space }

    val playing: Tab? get() = tabs.find { it.media?.playing == true } ?: tabs.find { it.media != null }

    init {
        engine.install(::connect)
        followSettings()
        updates.checkIfDue()
    }

    fun load(input: String) {
        if (input.isBlank()) return
        val address = clean(resolve(input, settings.searchEngine))
        (tab ?: open()).load(address)
    }

    fun open(space: Space = this.space): Tab = adopt(space).also { it.session.open(engine.runtime) }

    fun openPrivate(): Tab = open(Space.PRIVATE)

    fun adopt(space: Space): Tab {
        val tab = Tab(nextTabId++, space, engine.session(space), this)
        extension?.let(tab::connect)
        tabs += tab
        select(tab)
        return tab
    }

    fun select(tab: Tab) {
        this.tab?.takeIf { it != tab }?.session?.exitFullScreen()
        space = tab.space
        selected[tab.space.id] = tab
    }

    fun close(tab: Tab) {
        val siblings = tabs.filter { it.space == tab.space }
        val neighbour = siblings.getOrNull(siblings.indexOf(tab) - 1) ?: siblings.getOrNull(1)
        tabs -= tab
        tab.session.close()
        if (selected[tab.space.id] != tab) return
        if (neighbour == null) selected.remove(tab.space.id) else selected[tab.space.id] = neighbour
    }

    fun neighbour(step: Int): Tab? {
        val siblings = spaceTabs
        return siblings.getOrNull(siblings.indexOf(tab) + step)
    }

    fun clearBrowsingData() {
        history.clear()
        spaces.profiles.forEach(engine::clear)
    }

    fun switch(space: Space) {
        this.space = space
    }

    fun addSpace(name: String, profile: Profile.Named?) = switch(spaces.add(name, profile))

    fun renameSpace(space: Space, name: String) {
        val renamed = spaces.rename(space, name)
        tabs.filter { it.space == space }.forEach { it.space = renamed }
        if (this.space == space) switch(renamed)
    }

    fun removeSpace(space: Space) {
        tabs.filter { it.space == space }.forEach(::close)
        spaces.remove(space)
        if (this.space == space) switch(spaces.saved.first())
    }

    fun clean(url: String): String = Filter.clean(redirect(url) ?: url, settings.privacy)

    fun redirect(url: String): String? = settings.focus.redirect(url)

    fun page(): JSONObject =
        JSONObject().put("kind", "page").put("dark", settings.forceDark).put("backgroundPlay", settings.backgroundPlay)

    fun background() {
        if (!settings.backgroundPlay) tabs.forEach { it.media?.controller?.pause() }
    }

    private fun connect(ready: WebExtension) {
        extension = ready
        tabs.forEach { it.connect(ready) }
    }

    private fun followSettings() {
        scope.launch {
            snapshotFlow { settings.privacy }.drop(1).collect { privacy ->
                engine.protect(privacy)
                tabs.forEach { it.session.settings.useTrackingProtection = privacy.trackingProtection }
            }
        }
        scope.launch { snapshotFlow { settings.theme }.drop(1).collect(engine::follow) }
        scope.launch { snapshotFlow { settings.forceDark }.drop(1).collect { dark -> tabs.forEach { it.darken(dark) } } }
        scope.launch { snapshotFlow { settings.focus }.drop(1).collect(engine::focus) }
        scope.launch { snapshotFlow { settings.backgroundPlay }.drop(1).collect { enabled -> tabs.forEach { it.playInBackground(enabled) } } }
        scope.launch {
            snapshotFlow { settings.backgroundPlay && playing?.media?.playing == true }.filter { it }.collect { Playback.start(context) }
        }
    }
}
