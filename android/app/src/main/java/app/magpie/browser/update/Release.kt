package app.magpie.browser.update

import org.json.JSONObject

data class Download(val name: String, val url: String, val size: Long, val sha256: String?)

data class Offer(val version: String, val page: String, val notes: String, val download: Download?)

sealed interface Phase {
    data object Idle : Phase

    data object Checking : Phase

    data object UpToDate : Phase

    data class Failed(val message: String) : Phase

    data class Available(val offer: Offer) : Phase

    data class Downloading(val offer: Offer, val percent: Int) : Phase

    data class DownloadFailed(val offer: Offer, val message: String) : Phase

    data class Installing(val offer: Offer) : Phase
}

private const val UNREADABLE = "The update check gave an unreadable answer"

private fun JSONObject.text(key: String): String? = if (isNull(key)) null else getString(key)

private fun download(json: JSONObject?): Download? =
    json?.let { Download(it.getString("name"), it.getString("url"), it.getLong("size"), it.text("sha256")) }

private fun offer(json: JSONObject): Offer =
    Offer(json.getString("version"), json.getString("page"), json.optString("notes"), download(json.optJSONObject("download")))

fun parseStatus(json: String): Phase {
    val reply = JSONObject(json)
    return when (reply.optString("state")) {
        "up_to_date" -> Phase.UpToDate
        "available" -> Phase.Available(offer(reply))
        "failed" -> Phase.Failed(reply.optString("message", UNREADABLE))
        else -> Phase.Failed(UNREADABLE)
    }
}

val Phase.offer: Offer?
    get() = when (this) {
        is Phase.Available -> offer
        is Phase.Downloading -> offer
        is Phase.DownloadFailed -> offer
        is Phase.Installing -> offer
        Phase.Idle, Phase.Checking, Phase.UpToDate, is Phase.Failed -> null
    }

fun Phase.summary(): String = when (this) {
    Phase.Idle -> "Magpie looks for new releases on GitHub"
    Phase.Checking -> "Checking for updates…"
    Phase.UpToDate -> "Magpie is up to date"
    is Phase.Failed -> "Could not check for updates. $message"
    is Phase.Available -> "Update available: v${offer.version}"
    is Phase.Downloading -> "Downloading v${offer.version}… $percent%"
    is Phase.DownloadFailed -> "Could not install v${offer.version}. $message"
    is Phase.Installing -> "Finish installing v${offer.version} in the installer"
}
