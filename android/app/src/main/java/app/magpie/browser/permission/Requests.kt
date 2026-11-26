package app.magpie.browser.permission

import androidx.compose.runtime.mutableStateListOf
import app.magpie.browser.favicon.Origin
import org.mozilla.geckoview.GeckoResult
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoSession.PermissionDelegate.ContentPermission
import org.mozilla.geckoview.GeckoSession.PermissionDelegate.MediaSource

class Request(val origin: Origin, val kinds: Set<Kind>, private val memory: Permissions?, private val settle: (Request, Decision) -> Unit) {
    val rememberable: Boolean get() = memory != null

    fun answer(decision: Decision) = settle(this, decision)

    fun remember(decision: Decision) {
        memory?.remember(origin, kinds, decision)
        answer(decision)
    }
}

class SystemRequest(val permissions: Array<String>, private val callback: GeckoSession.PermissionDelegate.Callback, private val done: (SystemRequest) -> Unit) {
    fun grant() {
        done(this)
        callback.grant()
    }

    fun reject() {
        done(this)
        callback.reject()
    }
}

private fun contentValue(decision: Decision): Int =
    if (decision == Decision.Allow) ContentPermission.VALUE_ALLOW else ContentPermission.VALUE_DENY

private fun MediaSource.mediaKind(): Kind? = when (source) {
    MediaSource.SOURCE_CAMERA -> Kind.Camera
    MediaSource.SOURCE_MICROPHONE -> Kind.Microphone
    else -> null
}

class Requests(private val memory: Permissions?) : GeckoSession.PermissionDelegate {
    private val sites = mutableStateListOf<Request>()
    private val systems = mutableStateListOf<SystemRequest>()

    val site: Request? get() = sites.firstOrNull()

    val system: SystemRequest? get() = systems.firstOrNull()

    override fun onContentPermissionRequest(session: GeckoSession, permission: ContentPermission): GeckoResult<Int>? {
        if (permission.permission == GeckoSession.PermissionDelegate.PERMISSION_AUTOPLAY_INAUDIBLE) {
            return GeckoResult.fromValue(ContentPermission.VALUE_ALLOW)
        }
        val kind = Kind.of(permission.permission) ?: return super.onContentPermissionRequest(session, permission)
        val result = GeckoResult<Int>()
        ask(permission.uri, setOf(kind)) { decision -> result.complete(contentValue(decision)) }
        return result
    }

    override fun onMediaPermissionRequest(
        session: GeckoSession,
        uri: String,
        video: Array<out MediaSource>?,
        audio: Array<out MediaSource>?,
        callback: GeckoSession.PermissionDelegate.MediaCallback,
    ) {
        val camera = video?.find { it.mediaKind() == Kind.Camera }
        val microphone = audio?.find { it.mediaKind() == Kind.Microphone }
        val kinds = listOfNotNull(camera, microphone).mapNotNull { it.mediaKind() }.toSet()
        if (kinds.isEmpty()) return callback.reject()
        ask(uri, kinds) { decision -> if (decision == Decision.Allow) callback.grant(camera, microphone) else callback.reject() }
    }

    override fun onAndroidPermissionsRequest(
        session: GeckoSession,
        permissions: Array<out String>?,
        callback: GeckoSession.PermissionDelegate.Callback,
    ) {
        val needed = permissions?.takeIf { it.isNotEmpty() } ?: return callback.grant()
        systems += SystemRequest(arrayOf(*needed), callback, systems::remove)
    }

    private fun ask(uri: String, kinds: Set<Kind>, answer: (Decision) -> Unit) {
        val origin = Origin.of(uri) ?: return answer(Decision.Block)
        memory?.decision(origin, kinds)?.let { return answer(it) }
        sites += Request(origin, kinds, memory) { request, decision ->
            sites -= request
            answer(decision)
        }
    }
}
