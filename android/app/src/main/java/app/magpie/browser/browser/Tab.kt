package app.magpie.browser.browser

import android.content.pm.ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
import android.content.pm.ActivityInfo.SCREEN_ORIENTATION_USER_LANDSCAPE
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import app.magpie.browser.favicon.retention
import app.magpie.browser.media.Media
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import org.json.JSONArray
import org.json.JSONObject
import org.mozilla.geckoview.AllowOrDeny
import org.mozilla.geckoview.GeckoResult
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoSession.NavigationDelegate.LoadRequest
import org.mozilla.geckoview.GeckoSession.PermissionDelegate.ContentPermission
import org.mozilla.geckoview.GeckoSession.ProgressDelegate.SecurityInformation
import org.mozilla.geckoview.GeckoView
import org.mozilla.geckoview.MediaSession
import org.mozilla.geckoview.WebExtension
import org.mozilla.geckoview.WebResponse

const val PROGRESS_DONE = 100

private const val BLANK = "about:blank"

private fun strings(array: JSONArray): List<String> = (0 until array.length()).map(array::getString)

class Tab(val id: Long, space: Space, val session: GeckoSession, private val browser: Browser) :
    GeckoSession.NavigationDelegate,
    GeckoSession.ProgressDelegate,
    GeckoSession.ContentDelegate,
    GeckoSession.HistoryDelegate,
    MediaSession.Delegate,
    WebExtension.MessageDelegate,
    WebExtension.PortDelegate {

    var space by mutableStateOf(space)
    var url by mutableStateOf(""); private set
    var title by mutableStateOf(""); private set
    var secure by mutableStateOf(false); private set
    var progress by mutableIntStateOf(PROGRESS_DONE); private set
    var canGoBack by mutableStateOf(false); private set
    var canGoForward by mutableStateOf(false); private set
    var preview by mutableStateOf<ImageBitmap?>(null); private set
    var zoom by mutableStateOf(Zoom.DEFAULT); private set
    var media by mutableStateOf<Media?>(null); private set
    var fullScreen by mutableStateOf(false); private set
    private var portraitVideo by mutableStateOf(false)
    private var port: WebExtension.Port? = null

    private val remembered = space.profile != Profile.Private

    init {
        session.navigationDelegate = this
        session.progressDelegate = this
        session.contentDelegate = this
        session.mediaSessionDelegate = this
        if (remembered) session.historyDelegate = this
    }

    val blank: Boolean get() = url.isEmpty() || url == BLANK

    val label: String get() = title.ifBlank { url.ifBlank { "New tab" } }

    val orientation: Int
        get() = if (fullScreen && !portraitVideo) SCREEN_ORIENTATION_USER_LANDSCAPE else SCREEN_ORIENTATION_UNSPECIFIED

    fun load(address: String) {
        url = address
        session.loadUri(address)
    }

    suspend fun capture(view: GeckoView) {
        if (blank || view.session != session) return
        preview = snapshot(view) ?: return
    }

    fun connect(extension: WebExtension) = session.webExtensionController.setMessageDelegate(extension, this, NATIVE_APP)

    fun toggleReader() = send(JSONObject().put("kind", "reader"))

    fun darken(enabled: Boolean) = send(JSONObject().put("kind", "dark").put("enabled", enabled))

    fun playInBackground(enabled: Boolean) = send(JSONObject().put("kind", "background").put("enabled", enabled))

    fun zoom(step: Step) {
        zoom = zoom.step(step)
        send(JSONObject().put("kind", "zoom").put("zoom", zoom.factor))
    }

    private fun send(message: JSONObject) {
        port?.postMessage(message)
    }

    private fun replace(address: String) {
        session.load(GeckoSession.Loader().uri(address).flags(GeckoSession.LOAD_FLAGS_REPLACE_HISTORY))
    }

    override fun onLocationChange(
        session: GeckoSession,
        url: String?,
        perms: MutableList<ContentPermission>,
        hasUserGesture: Boolean,
    ) {
        this.url = url.orEmpty()
        browser.redirect(this.url)?.let(::replace)
    }

    override fun onCanGoBack(session: GeckoSession, canGoBack: Boolean) {
        this.canGoBack = canGoBack
    }

    override fun onCanGoForward(session: GeckoSession, canGoForward: Boolean) {
        this.canGoForward = canGoForward
    }

    override fun onLoadRequest(session: GeckoSession, request: LoadRequest): GeckoResult<AllowOrDeny> {
        val cleaned = browser.clean(request.uri)
        if (cleaned == request.uri) return GeckoResult.allow()
        session.loadUri(cleaned)
        return GeckoResult.deny()
    }

    override fun onNewSession(session: GeckoSession, uri: String): GeckoResult<GeckoSession> =
        GeckoResult.fromValue(browser.adopt(space).session)

    override fun onProgressChange(session: GeckoSession, progress: Int) {
        this.progress = progress
    }

    override fun onSecurityChange(session: GeckoSession, securityInfo: SecurityInformation) {
        secure = securityInfo.isSecure
    }

    override fun onTitleChange(session: GeckoSession, title: String?) {
        this.title = title.orEmpty()
        if (remembered && !blank) browser.history.title(url, this.title)
    }

    override fun onCloseRequest(session: GeckoSession) = browser.close(this)

    override fun onFullScreen(session: GeckoSession, fullScreen: Boolean) {
        this.fullScreen = fullScreen
    }

    override fun onExternalResponse(session: GeckoSession, response: WebResponse) = browser.downloads.save(response)

    override fun onVisited(session: GeckoSession, url: String, lastVisitedURL: String?, flags: Int): GeckoResult<Boolean> {
        val topLevel = flags and GeckoSession.HistoryDelegate.VISIT_TOP_LEVEL != 0
        if (topLevel) browser.history.record(url)
        return GeckoResult.fromValue(topLevel)
    }

    override fun onActivated(session: GeckoSession, mediaSession: MediaSession) {
        media = Media(mediaSession)
    }

    override fun onDeactivated(session: GeckoSession, mediaSession: MediaSession) {
        media = null
    }

    override fun onMetadata(session: GeckoSession, mediaSession: MediaSession, meta: MediaSession.Metadata) {
        media = media?.copy(title = meta.title.orEmpty(), artist = meta.artist.orEmpty())
    }

    override fun onPlay(session: GeckoSession, mediaSession: MediaSession) {
        media = media?.copy(playing = true)
    }

    override fun onPause(session: GeckoSession, mediaSession: MediaSession) {
        media = media?.copy(playing = false)
    }

    override fun onStop(session: GeckoSession, mediaSession: MediaSession) {
        media = media?.copy(playing = false)
    }

    override fun onFullscreen(
        session: GeckoSession,
        mediaSession: MediaSession,
        enabled: Boolean,
        meta: MediaSession.ElementMetadata?,
    ) {
        portraitVideo = enabled && meta != null && meta.height > meta.width
    }

    override fun onMessage(nativeApp: String, message: Any, sender: WebExtension.MessageSender): GeckoResult<Any> =
        browser.skips(message.toString())

    override fun onConnect(port: WebExtension.Port) {
        this.port = port
        port.setDelegate(this)
    }

    override fun onPortMessage(message: Any, port: WebExtension.Port) {
        val json = message as? JSONObject ?: return
        when (json.optString("kind")) {
            "page" -> port.postMessage(browser.page().put("zoom", zoom.factor))
            "icons" -> browser.favicons.learn(json.getString("page"), strings(json.getJSONArray("urls")), retention)
        }
    }

    override fun onDisconnect(port: WebExtension.Port) {
        if (this.port == port) this.port = null
    }
}
