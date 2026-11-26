package app.magpie.browser.permission

import org.mozilla.geckoview.GeckoSession.PermissionDelegate

enum class Kind(val purpose: String) {
    Camera("use your camera"),
    Microphone("use your microphone"),
    Location("know your location"),
    Notifications("send you notifications"),
    Storage("keep its data on this device"),
    Autoplay("play media with sound");

    companion object {
        fun of(permission: Int): Kind? = when (permission) {
            PermissionDelegate.PERMISSION_GEOLOCATION -> Location
            PermissionDelegate.PERMISSION_DESKTOP_NOTIFICATION -> Notifications
            PermissionDelegate.PERMISSION_PERSISTENT_STORAGE -> Storage
            PermissionDelegate.PERMISSION_AUTOPLAY_AUDIBLE -> Autoplay
            else -> null
        }
    }
}

fun purpose(kinds: Set<Kind>): String = kinds.sorted().joinToString(" and ") { it.purpose }
