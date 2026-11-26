package app.magpie.browser.history

import android.content.SharedPreferences
import androidx.compose.runtime.mutableStateListOf
import androidx.core.content.edit
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

data class Visit(val url: String, val title: String)

private const val HISTORY = "history"
private const val MAX_VISITS = 500

class History(private val preferences: SharedPreferences) {
    val visits = mutableStateListOf<Visit>().apply { addAll(load()) }

    fun record(url: String) {
        val title = visits.find { it.url == url }?.title.orEmpty()
        visits.removeAll { it.url == url }
        visits.add(0, Visit(url, title))
        if (visits.size > MAX_VISITS) visits.removeRange(MAX_VISITS, visits.size)
        save()
    }

    fun title(url: String, title: String) {
        val index = visits.indexOfFirst { it.url == url }
        if (index < 0) return
        visits[index] = visits[index].copy(title = title)
        save()
    }

    fun clear() {
        visits.clear()
        save()
    }

    private fun load(): List<Visit> = try {
        val array = JSONArray(preferences.getString(HISTORY, "[]"))
        (0 until array.length()).map { array.getJSONObject(it) }.map { Visit(it.getString("url"), it.optString("title")) }
    } catch (corrupt: JSONException) {
        emptyList()
    }

    private fun save() {
        val array = JSONArray(visits.map { JSONObject().put("url", it.url).put("title", it.title) })
        preferences.edit { putString(HISTORY, array.toString()) }
    }
}
