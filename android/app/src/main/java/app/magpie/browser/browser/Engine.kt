package app.magpie.browser.browser

import android.content.Context
import app.magpie.browser.focus.Focus
import app.magpie.browser.privacy.Filter
import app.magpie.browser.privacy.Privacy
import app.magpie.browser.settings.Settings
import app.magpie.browser.settings.ThemeMode
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import org.mozilla.geckoview.ContentBlocking
import org.mozilla.geckoview.GeckoRuntime
import org.mozilla.geckoview.GeckoRuntimeSettings
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoSessionSettings
import org.mozilla.geckoview.WebExtension
import java.io.File
import java.util.Base64

const val NATIVE_APP = "magpie"

private const val EXTENSION_URL = "resource://android/assets/extension/"
private const val EXTENSION_ID = "magpie@magpie.app"
private const val COOKIES = ContentBlocking.CookieBehavior.ACCEPT_FIRST_PARTY_AND_ISOLATE_OTHERS

private val COLOR_SCHEMES = mapOf(
    ThemeMode.System to GeckoRuntimeSettings.COLOR_SCHEME_SYSTEM,
    ThemeMode.Light to GeckoRuntimeSettings.COLOR_SCHEME_LIGHT,
    ThemeMode.Dark to GeckoRuntimeSettings.COLOR_SCHEME_DARK,
)

private fun antiTracking(privacy: Privacy): Int =
    if (privacy.trackingProtection) ContentBlocking.AntiTracking.STRICT else ContentBlocking.AntiTracking.NONE

private fun protectionLevel(privacy: Privacy): Int =
    if (privacy.trackingProtection) ContentBlocking.EtpLevel.STRICT else ContentBlocking.EtpLevel.NONE

private fun contentBlocking(privacy: Privacy): ContentBlocking.Settings =
    ContentBlocking.Settings.Builder()
        .antiTracking(antiTracking(privacy))
        .enhancedTrackingProtectionLevel(protectionLevel(privacy))
        .cookieBehavior(COOKIES)
        .cookieBehaviorPrivateMode(COOKIES)
        .safeBrowsing(ContentBlocking.SafeBrowsing.DEFAULT)
        .build()

private fun blockingMessage(privacy: Privacy): JSONObject =
    JSONObject().put("kind", "blocking").put("enabled", privacy.adBlock)

private fun focusMessage(focus: Focus): JSONObject =
    JSONObject().put("kind", "focus").put("script", focus.script()).put("update", focus.update())

private fun snapshotMessage(snapshot: ByteArray): JSONObject =
    JSONObject().put("kind", "snapshot").put("snapshot", Base64.getEncoder().encodeToString(snapshot))

class Engine(context: Context, private val settings: Settings, private val scope: CoroutineScope) :
    WebExtension.MessageDelegate,
    WebExtension.PortDelegate {

    private val dataDir = context.filesDir
    private var blocker: WebExtension.Port? = null

    val runtime: GeckoRuntime = GeckoRuntime.create(
        context,
        GeckoRuntimeSettings.Builder()
            .contentBlocking(contentBlocking(settings.privacy))
            .preferredColorScheme(COLOR_SCHEMES.getValue(settings.theme))
            .build(),
    )

    fun session(space: Space): GeckoSession {
        val builder = GeckoSessionSettings.Builder().useTrackingProtection(settings.privacy.trackingProtection)
        when (val profile = space.profile) {
            is Profile.Named -> builder.contextId(profile.id)
            Profile.Private -> builder.usePrivateMode(true)
        }
        return GeckoSession(builder.build())
    }

    fun install(onReady: (WebExtension) -> Unit) {
        val controller = runtime.webExtensionController
        controller.ensureBuiltIn(EXTENSION_URL, EXTENSION_ID)
            .then { extension -> controller.setAllowedInPrivateBrowsing(checkNotNull(extension), true) }
            .accept { extension ->
                val ready = checkNotNull(extension)
                ready.setMessageDelegate(this, NATIVE_APP)
                onReady(ready)
            }
    }

    fun protect(privacy: Privacy) {
        runtime.settings.contentBlocking
            .setAntiTracking(antiTracking(privacy))
            .setEnhancedTrackingProtectionLevel(protectionLevel(privacy))
        blocker?.postMessage(blockingMessage(privacy))
    }

    fun focus(focus: Focus) {
        blocker?.postMessage(focusMessage(focus))
    }

    fun follow(theme: ThemeMode) {
        runtime.settings.preferredColorScheme = COLOR_SCHEMES.getValue(theme)
    }

    fun clear(profile: Profile.Named) {
        runtime.storageController.clearDataForSessionContext(profile.id)
    }

    private suspend fun sendSnapshot(load: (File) -> ByteArray?) {
        val message = withContext(Dispatchers.IO) { load(dataDir)?.let(::snapshotMessage) } ?: return
        blocker?.postMessage(message)
    }

    override fun onConnect(port: WebExtension.Port) {
        blocker = port
        port.setDelegate(this)
        port.postMessage(blockingMessage(settings.privacy))
        port.postMessage(focusMessage(settings.focus))
        scope.launch {
            sendSnapshot(Filter::cachedSnapshot)
            sendSnapshot(Filter::refreshedSnapshot)
        }
    }

    override fun onDisconnect(port: WebExtension.Port) {
        if (blocker == port) blocker = null
    }
}
