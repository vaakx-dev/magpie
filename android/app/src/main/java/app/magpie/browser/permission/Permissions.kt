package app.magpie.browser.permission

import android.content.SharedPreferences
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.core.content.edit
import app.magpie.browser.favicon.Origin

private const val SITE_PERMISSIONS = "site_permissions"

class Permissions(private val preferences: SharedPreferences) {
    private var decisions by mutableStateOf(Decisions.stored(preferences.getString(SITE_PERMISSIONS, null)))

    val origins: List<Origin> get() = decisions.origins

    fun site(origin: Origin): Map<Kind, Decision> = decisions.site(origin)

    fun decision(origin: Origin, kinds: Set<Kind>): Decision? = decisions.decision(origin, kinds)

    fun remember(origin: Origin, kinds: Set<Kind>, decision: Decision) = save(decisions.with(origin, kinds, decision))

    fun forget(origin: Origin) = save(decisions.without(origin))

    fun clear() = save(Decisions())

    private fun save(changed: Decisions) {
        decisions = changed
        preferences.edit { putString(SITE_PERMISSIONS, changed.encoded()) }
    }
}
