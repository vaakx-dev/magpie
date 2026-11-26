package app.magpie.browser.permission

import org.mozilla.geckoview.GeckoSession.PermissionDelegate

enum class Kind(val label: String, val purpose: String) {
    Camera("Camera", "use your camera"),
    Microphone("Microphone", "use your microphone"),
    Location("Location", "know your location"),
    Notifications("Notifications", "send you notifications"),
    Storage("Storage", "keep its data on this device"),
    Autoplay("Autoplay with sound", "play media with sound");

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

fun summary(site: Map<Kind, Decision>): String =
    site.entries.sortedBy { it.key }.joinToString(" · ") { (kind, decision) -> "${kind.label} ${decision.label}" }
