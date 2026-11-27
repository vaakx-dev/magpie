package app.magpie.browser.dark

import app.magpie.browser.site.Choices
import app.magpie.browser.site.Site
import app.magpie.browser.storage.Retention
import org.json.JSONObject

data class ForceDark(val enabled: Boolean = false, val sites: Choices<Boolean> = Choices()) {
    fun on(site: Site?, retention: Retention): Boolean = site?.let { sites.of(it, retention) } ?: enabled

    fun chosen(site: Site, retention: Retention): Boolean = sites.of(site, retention) != null

    fun toggle(site: Site, retention: Retention): ForceDark {
        val next = !on(site, retention)
        return copy(sites = sites.with(site, next.takeUnless { it == enabled }, retention))
    }

    fun forgetPrivate(): ForceDark = copy(sites = sites.forgetMemory())

    fun encodedSites(): String = JSONObject(sites.disk.mapKeys { it.key.name }).toString()

    companion object {
        fun stored(enabled: Boolean, sites: String?): ForceDark {
            val json = JSONObject(sites ?: "{}")
            return ForceDark(enabled, Choices(json.keys().asSequence().associate { Site(it) to json.getBoolean(it) }))
        }
    }
}
