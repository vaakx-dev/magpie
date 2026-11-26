package app.magpie.browser.browser

import app.magpie.browser.storage.JsonFile
import org.json.JSONObject
import java.net.URI

private const val WWW = "www."
private val WEB_SCHEMES = setOf("http", "https")

fun site(url: String): String? {
    val uri = runCatching { URI(url) }.getOrNull() ?: return null
    if (uri.scheme?.lowercase() !in WEB_SCHEMES) return null
    return uri.host?.lowercase()?.removePrefix(WWW)
}

data class SiteZooms(val levels: Map<String, Zoom> = emptyMap()) {
    fun of(url: String): Zoom = site(url)?.let(levels::get) ?: Zoom.DEFAULT

    fun with(url: String, zoom: Zoom): SiteZooms {
        val site = site(url) ?: return this
        return SiteZooms(if (zoom == Zoom.DEFAULT) levels - site else levels + (site to zoom))
    }

    fun json(): JSONObject = JSONObject(levels.mapValues { it.value.percent })

    companion object {
        fun from(json: JSONObject): SiteZooms = SiteZooms(json.keys().asSequence().associateWith { Zoom(json.getInt(it)) })
    }
}

class Zooms private constructor(private val file: JsonFile?, private var saved: SiteZooms) {
    constructor(file: JsonFile) : this(file, file.read(SiteZooms::from) ?: SiteZooms())

    fun of(url: String): Zoom = saved.of(url)

    fun set(url: String, zoom: Zoom) {
        val changed = saved.with(url, zoom)
        if (changed == saved) return
        saved = changed
        file?.write(changed.json())
    }

    fun flush() {
        file?.flush()
    }

    fun inMemory(): Zooms = Zooms(null, saved)
}
