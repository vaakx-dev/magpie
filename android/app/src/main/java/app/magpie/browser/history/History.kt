package app.magpie.browser.history

import android.content.SharedPreferences
import androidx.compose.runtime.mutableStateListOf
import androidx.core.content.edit
import app.magpie.browser.storage.JsonFile
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

data class Visit(val url: String, val title: String, val visited: Long)

enum class Day(val label: String) { Today("Today"), Yesterday("Yesterday"), Earlier("Earlier") }

data class Group(val day: Day, val visits: List<Visit>)

private const val LEGACY_KEY = "history"
private const val VISITS = "visits"
private const val URL = "url"
private const val TITLE = "title"
private const val VISITED = "visited"
private const val UNKNOWN_TIME = 0L
const val MAX_VISITS = 5_000

private fun Visit.json(): JSONObject = JSONObject().put(URL, url).put(TITLE, title).put(VISITED, visited)

private fun visit(json: JSONObject): Visit = Visit(json.getString(URL), json.optString(TITLE), json.optLong(VISITED, UNKNOWN_TIME))

private fun visits(array: JSONArray): List<Visit> = (0 until array.length()).map { visit(array.getJSONObject(it)) }

private fun decode(json: JSONObject): List<Visit> = visits(json.getJSONArray(VISITS))

private fun encode(visits: List<Visit>): JSONObject = JSONObject().put(VISITS, JSONArray(visits.map { it.json() }))

private fun legacy(preferences: SharedPreferences): List<Visit> = try {
    visits(JSONArray(preferences.getString(LEGACY_KEY, "[]")))
} catch (corrupt: JSONException) {
    emptyList()
}

private fun date(time: Long, zone: ZoneId): LocalDate = Instant.ofEpochMilli(time).atZone(zone).toLocalDate()

private fun day(visit: Visit, today: LocalDate, zone: ZoneId): Day {
    val date = date(visit.visited, zone)
    return when {
        !date.isBefore(today) -> Day.Today
        date == today.minusDays(1) -> Day.Yesterday
        else -> Day.Earlier
    }
}

fun matching(visits: List<Visit>, query: String): List<Visit> {
    val text = query.trim()
    return visits.filter { it.url.contains(text, ignoreCase = true) || it.title.contains(text, ignoreCase = true) }
}

fun grouped(visits: List<Visit>, now: Long, zone: ZoneId = ZoneId.systemDefault()): List<Group> {
    val today = date(now, zone)
    return visits.groupBy { day(it, today, zone) }.map { (day, visits) -> Group(day, visits) }
}

class History(private val file: JsonFile, preferences: SharedPreferences) {
    val visits = mutableStateListOf<Visit>().apply { addAll(file.read(::decode) ?: migrate(preferences)) }

    fun record(url: String) {
        val title = visits.find { it.url == url }?.title.orEmpty()
        visits.removeAll { it.url == url }
        visits.add(0, Visit(url, title, System.currentTimeMillis()))
        if (visits.size > MAX_VISITS) visits.removeRange(MAX_VISITS, visits.size)
        save()
    }

    fun title(url: String, title: String) {
        val index = visits.indexOfFirst { it.url == url }
        if (index < 0) return
        visits[index] = visits[index].copy(title = title)
        save()
    }

    fun forget(url: String) {
        if (visits.removeAll { it.url == url }) save()
    }

    fun clear() {
        visits.clear()
        save()
    }

    fun flush() {
        file.flush()
    }

    private fun migrate(preferences: SharedPreferences): List<Visit> {
        if (!preferences.contains(LEGACY_KEY)) return emptyList()
        val migrated = legacy(preferences).take(MAX_VISITS)
        file.write(encode(migrated))
        if (file.flush()) preferences.edit { remove(LEGACY_KEY) }
        return migrated
    }

    private fun save() {
        file.write(encode(visits))
    }
}
