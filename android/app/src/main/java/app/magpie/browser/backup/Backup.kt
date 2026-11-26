package app.magpie.browser.backup

import app.magpie.browser.favorite.Favorite
import app.magpie.browser.favorite.favorites
import app.magpie.browser.favorite.json
import app.magpie.browser.settings.Preference
import app.magpie.browser.space.Space
import app.magpie.browser.space.json
import app.magpie.browser.space.spaces
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

private const val APP = "magpie"
private const val FORMAT = 1
private const val PLATFORM = "android"
private const val BYTE_ORDER_MARK = "\uFEFF"
private const val INDENT_SPACES = 2
private const val NOT_MAGPIE = "This file isn't a Magpie backup."
private const val FROM_DESKTOP = "This backup is from Magpie for desktop. Restore it there; Android can't read desktop settings."

data class Backup(val settings: Map<String, Preference>, val spaces: List<Space>, val favorites: List<Favorite>) {
    fun json(): String = JSONObject()
        .put("app", APP)
        .put("format", FORMAT)
        .put("platform", PLATFORM)
        .put("settings", JSONObject(settings.mapValues { it.value.json() }))
        .put("spaces", JSONArray(spaces.map { it.json() }))
        .put("favorites", JSONArray(favorites.map { it.json() }))
        .toString(INDENT_SPACES)
}

sealed interface Parsed {
    data class Valid(val backup: Backup) : Parsed

    data class Invalid(val problem: String) : Parsed
}

private fun Preference.json(): Any = when (this) {
    is Preference.Text -> text
    is Preference.Switch -> on
    is Preference.Texts -> JSONArray(texts.sorted())
}

private fun preference(key: String, value: Any): Preference = when (value) {
    is String -> Preference.Text(value)
    is Boolean -> Preference.Switch(value)
    is JSONArray -> Preference.Texts((0 until value.length()).map(value::getString).toSet())
    else -> throw JSONException("setting $key has an unexpected value")
}

private fun preferences(json: JSONObject): Map<String, Preference> =
    json.keys().asSequence().associateWith { preference(it, json.get(it)) }

private fun problem(json: JSONObject): String? {
    val format = json.optInt("format")
    return when {
        json.optString("app") != APP -> NOT_MAGPIE
        format > FORMAT -> "This backup is from a newer Magpie (format $format). Update Magpie to restore it."
        json.optString("platform") != PLATFORM -> FROM_DESKTOP
        else -> null
    }
}

private fun decoded(json: JSONObject): Backup = Backup(
    settings = preferences(json.getJSONObject("settings")),
    spaces = spaces(json.getJSONArray("spaces")),
    favorites = favorites(json.getJSONArray("favorites")),
)

private fun json(text: String): JSONObject? = try {
    JSONObject(text.removePrefix(BYTE_ORDER_MARK))
} catch (unreadable: JSONException) {
    null
}

fun parse(text: String): Parsed {
    val json = json(text) ?: return Parsed.Invalid(NOT_MAGPIE)
    problem(json)?.let { return Parsed.Invalid(it) }
    return try {
        Parsed.Valid(decoded(json))
    } catch (damaged: JSONException) {
        Parsed.Invalid("This backup is damaged: ${damaged.message}")
    }
}
