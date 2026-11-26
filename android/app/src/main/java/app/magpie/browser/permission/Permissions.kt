package app.magpie.browser.permission

import android.content.SharedPreferences
import androidx.core.content.edit
import app.magpie.browser.favicon.Origin

private const val SITE_PERMISSIONS = "site_permissions"

class Permissions(private val preferences: SharedPreferences) {
    private var decisions = Decisions.stored(preferences.getString(SITE_PERMISSIONS, null))

    fun decision(origin: Origin, kinds: Set<Kind>): Decision? = decisions.decision(origin, kinds)

    fun remember(origin: Origin, kinds: Set<Kind>, decision: Decision) {
        decisions = decisions.with(origin, kinds, decision)
        preferences.edit { putString(SITE_PERMISSIONS, decisions.encoded()) }
    }
}
