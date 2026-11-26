package app.magpie.browser.favorite

import android.content.SharedPreferences
import androidx.compose.runtime.mutableStateListOf
import androidx.core.content.edit
import org.json.JSONArray
import org.json.JSONObject

data class Favorite(val url: String, val title: String)

private const val FAVORITES = "favorites"
private val DEFAULTS = listOf(
    Favorite("https://www.youtube.com", "YouTube"),
    Favorite("https://github.com/vaakx-dev/magpie", "Magpie"),
)

fun Favorite.json(): JSONObject = JSONObject().put("url", url).put("title", title)

fun favorites(array: JSONArray): List<Favorite> =
    (0 until array.length()).map { array.getJSONObject(it) }.map { Favorite(it.getString("url"), it.getString("title")) }

fun stored(json: String?): List<Favorite> = json?.let { favorites(JSONArray(it)) } ?: DEFAULTS

class Favorites(private val preferences: SharedPreferences) {
    val list = mutableStateListOf<Favorite>().apply { addAll(stored(preferences.getString(FAVORITES, null))) }

    fun contains(url: String): Boolean = list.any { it.url == url }

    fun toggle(url: String, title: String) {
        val existing = list.find { it.url == url }
        if (existing == null) list += Favorite(url, title.ifBlank { url }) else list -= existing
        save()
    }

    fun remove(favorite: Favorite) {
        list -= favorite
        save()
    }

    fun replace(favorite: Favorite, edited: Favorite) {
        list.removeAll { it.url == edited.url && it != favorite }
        list[list.indexOf(favorite)] = edited
        save()
    }

    fun restore(favorites: List<Favorite>) {
        list.clear()
        list.addAll(favorites.distinctBy(Favorite::url))
        save()
    }

    private fun save() {
        val array = JSONArray(list.map { it.json() })
        preferences.edit { putString(FAVORITES, array.toString()) }
    }
}
