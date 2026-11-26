package app.magpie.browser.browser

import org.json.JSONArray
import org.json.JSONObject

data class SavedTab(val url: String, val title: String)

data class SavedSpace(val id: String, val tabs: List<SavedTab>, val selected: Int?)

data class SavedTabs(val space: String?, val spaces: List<SavedSpace>)

private fun objects(array: JSONArray): List<JSONObject> = (0 until array.length()).map(array::getJSONObject)

private fun SavedTab.json(): JSONObject = JSONObject().put("url", url).put("title", title)

private fun SavedSpace.json(): JSONObject =
    JSONObject().put("id", id).put("tabs", JSONArray(tabs.map { it.json() })).putOpt("selected", selected)

private fun savedTab(json: JSONObject): SavedTab = SavedTab(json.getString("url"), json.getString("title"))

private fun savedSpace(json: JSONObject): SavedSpace = SavedSpace(
    json.getString("id"),
    objects(json.getJSONArray("tabs")).map(::savedTab),
    json.takeIf { it.has("selected") }?.getInt("selected"),
)

fun SavedTabs.json(): JSONObject = JSONObject().putOpt("space", space).put("spaces", JSONArray(spaces.map { it.json() }))

fun savedTabs(json: JSONObject): SavedTabs =
    SavedTabs(json.optString("space").ifEmpty { null }, objects(json.getJSONArray("spaces")).map(::savedSpace))
