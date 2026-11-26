package app.magpie.browser.dark

import app.magpie.browser.site.Site
import org.json.JSONObject

data class ForceDark(val enabled: Boolean = false, val sites: Map<Site, Boolean> = emptyMap()) {
    fun on(site: Site?): Boolean = site?.let(sites::get) ?: enabled

    fun toggle(site: Site): ForceDark {
        val next = !on(site)
        return copy(sites = if (next == enabled) sites - site else sites + (site to next))
    }

    fun reset(site: Site): ForceDark = copy(sites = sites - site)

    fun encodedSites(): String = JSONObject(sites.mapKeys { it.key.name }).toString()

    companion object {
        fun stored(enabled: Boolean, sites: String?): ForceDark {
            val json = JSONObject(sites ?: "{}")
            return ForceDark(enabled, json.keys().asSequence().associate { Site(it) to json.getBoolean(it) })
        }
    }
}
