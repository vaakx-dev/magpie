package app.magpie.browser.space

import android.content.SharedPreferences
import androidx.compose.runtime.mutableStateListOf
import androidx.core.content.edit
import org.json.JSONArray
import org.json.JSONObject
import java.util.UUID

private const val SPACES = "spaces"

private val DEFAULT_SPACES = listOf(
    Space("personal", "Personal", Profile.Named("personal", "Personal")),
    Space("work", "Work", Profile.Named("work", "Work")),
)

class Spaces(private val preferences: SharedPreferences) {
    val saved = mutableStateListOf<Space>().apply { addAll(load()) }

    val all: List<Space> get() = saved + Space.PRIVATE

    val profiles: List<Profile.Named> get() = saved.map { it.profile }.filterIsInstance<Profile.Named>().distinct()

    fun add(name: String, profile: Profile.Named?): Space {
        val space = Space(newId(), name, profile ?: Profile.Named(newId(), name))
        saved += space
        save()
        return space
    }

    fun rename(space: Space, name: String): Space {
        val renamed = space.copy(name = name)
        saved[saved.indexOf(space)] = renamed
        save()
        return renamed
    }

    fun remove(space: Space) {
        saved -= space
        save()
    }

    private fun newId(): String = UUID.randomUUID().toString()

    private fun load(): List<Space> {
        val stored = preferences.getString(SPACES, null) ?: return DEFAULT_SPACES
        val array = JSONArray(stored)
        return (0 until array.length()).map { array.getJSONObject(it).toSpace() }
    }

    private fun save() {
        val array = JSONArray(saved.map { it.toJson() })
        preferences.edit { putString(SPACES, array.toString()) }
    }
}

private fun Space.toJson(): JSONObject {
    val named = when (profile) {
        is Profile.Named -> profile
        Profile.Private -> error("Private spaces are never saved")
    }
    return JSONObject()
        .put("id", id)
        .put("name", name)
        .put("profileId", named.id)
        .put("profileLabel", named.label)
}

private fun JSONObject.toSpace(): Space =
    Space(getString("id"), getString("name"), Profile.Named(getString("profileId"), getString("profileLabel")))
