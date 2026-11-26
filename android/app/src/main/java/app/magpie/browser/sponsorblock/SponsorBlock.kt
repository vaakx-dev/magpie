package app.magpie.browser.sponsorblock

import app.magpie.browser.core.MagpieCore
import org.json.JSONArray
import org.json.JSONObject

const val ENABLED = "enabled"
const val SHOW_SKIP_NOTICE = "show_skip_notice"

data class CategorySwitch(val key: String, val label: String, val description: String, val on: Boolean)

data class View(val enabled: Boolean, val showSkipNotice: Boolean, val categories: List<CategorySwitch>, val channels: List<String>)

private fun categorySwitch(json: JSONObject) =
    CategorySwitch(json.getString("key"), json.getString("label"), json.getString("description"), json.getBoolean("on"))

private fun <T> JSONArray.map(read: JSONArray.(Int) -> T): List<T> = (0 until length()).map { read(it) }

fun parseView(json: String): View {
    val view = JSONObject(json)
    return View(
        enabled = view.getBoolean(ENABLED),
        showSkipNotice = view.getBoolean(SHOW_SKIP_NOTICE),
        categories = view.getJSONArray("categories").map { categorySwitch(getJSONObject(it)) },
        channels = view.getJSONArray("channels").map(JSONArray::getString),
    )
}

data class SponsorBlock(val config: String) {
    val view: View? by lazy { if (MagpieCore.loaded) parseView(MagpieCore.sponsorBlockView(config)) else null }

    fun switch(key: String, on: Boolean): SponsorBlock = SponsorBlock(MagpieCore.sponsorBlockSwitch(config, key, on))

    fun resume(channel: String): SponsorBlock = SponsorBlock(MagpieCore.sponsorBlockResume(config, channel))

    fun answer(request: String): JSONObject? =
        if (MagpieCore.loaded) MagpieCore.sponsorBlockAnswer(config, request)?.let(::JSONObject) else null
}
