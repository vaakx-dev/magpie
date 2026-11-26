package app.magpie.browser.browser

import android.content.Context
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import app.magpie.browser.address.resolve
import app.magpie.browser.backup.Backup
import app.magpie.browser.download.Downloads
import app.magpie.browser.external.Apps
import app.magpie.browser.favicon.Favicons
import app.magpie.browser.favorite.Favorites
import app.magpie.browser.history.History
import app.magpie.browser.media.Playback
import app.magpie.browser.password.Vault
import app.magpie.browser.permission.Permissions
import app.magpie.browser.privacy.Filter
import app.magpie.browser.settings.Settings
import app.magpie.browser.site.Site
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import app.magpie.browser.space.Spaces
import app.magpie.browser.storage.JsonFile
import app.magpie.browser.update.Updates
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import org.mozilla.geckoview.GeckoResult
import org.mozilla.geckoview.WebExtension

private const val PREFERENCES = "magpie"
private const val TABS_FILE = "tabs.json"
private const val ZOOMS_FILE = "zoom.json"

private val Tab.saveable: Boolean get() = space.profile is Profile.Named && !blank

class Browser(private val context: Context) {
    val scope = MainScope()
    private val preferences = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
    val settings = Settings(preferences)
    val spaces = Spaces(preferences)
    val favorites = Favorites(preferences)
    val history = History(preferences)
    val downloads = Downloads(context, scope)
    val favicons = Favicons(context, scope)
    val vault = Vault(context)
    val zooms = Zooms(JsonFile(context, ZOOMS_FILE, scope))
    var privateZooms = zooms.inMemory()
        private set
    val permissions = Permissions(preferences)
    val apps = Apps(context)
    val engine = Engine(context, settings, scope)
    val updates = Updates(context, scope, settings)

    val tabs = mutableStateListOf<Tab>()
    var space by mutableStateOf(spaces.saved.first())
        private set
    private val selected = mutableStateMapOf<String, Tab>()
    val closed = ClosedTabs()
    private val tabsFile = JsonFile(context, TABS_FILE, scope)
    private var extension: WebExtension? = null
    private var nextTabId = 0L

    val tab: Tab? get() = selected[space.id]

    val spaceTabs: List<Tab> get() = tabs.filter { it.space == space }

    val playing: Tab? get() = tabs.find { it.media?.playing == true } ?: tabs.find { it.media != null }

    init {
        engine.install(::connect)
        if (settings.privacy.clearOnExit) clearBrowsingData()
        restoreTabs()
        followTabs()
        followSettings()
        updates.checkIfDue()
    }

    fun load(input: String) {
        if (input.isBlank()) return
        val address = clean(resolve(input, settings.search))
        (tab ?: open()).load(address)
    }

    fun open(space: Space = this.space): Tab = adopt(space).also { it.session.open(engine.runtime) }

    fun openPrivate(): Tab = open(Space.PRIVATE)

    fun adopt(space: Space): Tab = create(space).also {
        tabs += it
        select(it)
    }

    fun select(tab: Tab) {
        this.tab?.takeIf { it != tab }?.session?.exitFullScreen()
        selected[tab.space.id] = tab
        switch(tab.space)
    }

    fun close(tab: Tab) {
        if (tab.saveable) closed.push(ClosedTab(tab.space.id, tab.saved, tabs.indexOf(tab)))
        discard(tab)
    }

    fun undo() {
        val last = closed.pop() ?: return
        val space = spaces.saved.find { it.id == last.space } ?: return
        select(restore(space, last.tab, last.index.coerceAtMost(tabs.size)))
    }

    fun neighbour(step: Int): Tab? {
        val siblings = spaceTabs
        return siblings.getOrNull(siblings.indexOf(tab) + step)
    }

    fun clearBrowsingData() {
        history.clear()
        favicons.clear()
        downloads.clear()
        closed.clear()
        permissions.clear()
        spaces.profiles.forEach(engine::clear)
    }

    fun toggleShield(tab: Tab, site: Site) {
        settings.privacy = settings.privacy.toggle(site)
        engine.protect(settings.privacy)
        tab.session.reload()
    }

    fun switch(space: Space) {
        this.space = space
        tab?.wake()
    }

    fun addSpace(name: String, profile: Profile.Named?) = switch(spaces.add(name, profile))

    fun renameSpace(space: Space, name: String) {
        val renamed = spaces.rename(space, name)
        tabs.filter { it.space == space }.forEach { it.space = renamed }
        if (this.space == space) switch(renamed)
    }

    fun removeSpace(space: Space) {
        tabs.filter { it.space == space }.forEach(::discard)
        closed.forget(space.id)
        spaces.remove(space)
        if (this.space == space) switch(spaces.saved.first())
    }

    fun backup(): Backup = Backup(settings.saved(), spaces.saved.toList(), favorites.list.toList())

    fun restore(backup: Backup) {
        settings.restore(backup.settings)
        favorites.restore(backup.favorites)
        backup.spaces.forEach(::merge)
    }

    fun clean(url: String): String = Filter.clean(redirect(url) ?: url, settings.privacy)

    fun redirect(url: String): String? = settings.focus.redirect(url)

    fun skips(request: String): GeckoResult<Any> {
        val sponsorBlock = settings.sponsorBlock
        val reply = GeckoResult<Any>()
        scope.launch { reply.complete(withContext(Dispatchers.IO) { sponsorBlock.answer(request) }) }
        return reply
    }

    fun dark(url: String): Boolean = settings.forceDark.on(Site.of(url))

    fun page(url: String): JSONObject =
        JSONObject().put("kind", "page").put("dark", dark(url)).put("backgroundPlay", settings.backgroundPlay)

    fun background() {
        if (!settings.backgroundPlay) tabs.forEach { it.media?.controller?.pause() }
    }

    private fun create(space: Space): Tab = Tab(nextTabId++, space, engine.session(space), this).also { tab ->
        extension?.let(tab::connect)
    }

    private fun restore(space: Space, saved: SavedTab, index: Int): Tab = create(space).also {
        it.restore(saved)
        tabs.add(index, it)
    }

    private fun discard(tab: Tab) {
        val siblings = tabs.filter { it.space == tab.space }
        val neighbour = siblings.getOrNull(siblings.indexOf(tab) - 1) ?: siblings.getOrNull(1)
        tabs -= tab
        tab.session.close()
        if (tabs.none { it.space == Space.PRIVATE }) privateZooms = zooms.inMemory()
        if (selected[tab.space.id] != tab) return
        if (neighbour == null) selected.remove(tab.space.id) else selected[tab.space.id] = neighbour
        this.tab?.wake()
    }

    private fun savedTabs(): SavedTabs =
        SavedTabs(space.id.takeIf { space.profile is Profile.Named }, spaces.saved.map(::savedSpace))

    private fun savedSpace(space: Space): SavedSpace {
        val kept = tabs.filter { it.space == space && it.saveable }
        val active = kept.indexOf(selected[space.id]).takeIf { it >= 0 }
        return SavedSpace(space.id, kept.map { it.saved }, active)
    }

    private fun restoreTabs() {
        val saved = tabsFile.read(::savedTabs) ?: return
        saved.spaces.forEach(::restoreSpace)
        switch(spaces.saved.find { it.id == saved.space } ?: space)
    }

    private fun merge(backedUp: Space) {
        val existing = spaces.saved.find { it.id == backedUp.id }
        if (existing == null) spaces.restore(backedUp) else renameSpace(existing, backedUp.name)
    }

    private fun restoreSpace(saved: SavedSpace) {
        val space = spaces.saved.find { it.id == saved.id } ?: return
        val restored = saved.tabs.map { restore(space, it, tabs.size) }
        val active = saved.selected?.let(restored::getOrNull) ?: restored.lastOrNull() ?: return
        selected[space.id] = active
    }

    private fun followTabs() {
        scope.launch { snapshotFlow { savedTabs() }.drop(1).collect { tabsFile.write(it.json()) } }
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
        scope.launch { snapshotFlow { settings.forceDark }.drop(1).collect { tabs.forEach { tab -> tab.darken(dark(tab.url)) } } }
        scope.launch { snapshotFlow { settings.focus }.drop(1).collect(engine::focus) }
        scope.launch { snapshotFlow { settings.backgroundPlay }.drop(1).collect { enabled -> tabs.forEach { it.playInBackground(enabled) } } }
        scope.launch {
            snapshotFlow { settings.backgroundPlay && playing?.media?.playing == true }.filter { it }.collect { Playback.start(context) }
        }
    }
}
