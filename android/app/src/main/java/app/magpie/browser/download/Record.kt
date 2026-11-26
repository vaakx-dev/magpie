package app.magpie.browser.download

import org.json.JSONArray
import org.json.JSONObject

data class Record(val name: String, val type: String, val total: Long?, val progress: Progress, val received: Long, val uri: String?)

private fun Record.json(): JSONObject = JSONObject()
    .put("name", name)
    .put("type", type)
    .putOpt("total", total)
    .put("progress", progress.name)
    .put("received", received)
    .putOpt("uri", uri)

private fun record(json: JSONObject): Record = Record(
    name = json.getString("name"),
    type = json.getString("type"),
    total = json.takeIf { it.has("total") }?.getLong("total"),
    progress = Progress.valueOf(json.getString("progress")),
    received = json.getLong("received"),
    uri = json.optString("uri").ifEmpty { null },
)

fun List<Record>.json(): JSONObject = JSONObject().put("downloads", JSONArray(map { it.json() }))

fun records(json: JSONObject): List<Record> {
    val array = json.getJSONArray("downloads")
    return (0 until array.length()).map(array::getJSONObject).map(::record)
}
