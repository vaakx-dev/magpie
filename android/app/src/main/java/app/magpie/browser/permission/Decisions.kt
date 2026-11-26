package app.magpie.browser.permission

import app.magpie.browser.favicon.Origin
import org.json.JSONObject

enum class Decision { Allow, Block }

private fun decodedSite(json: JSONObject): Map<Kind, Decision> =
    json.keys().asSequence().mapNotNull { key ->
        val kind = Kind.entries.find { it.name == key }
        val decision = Decision.entries.find { it.name == json.optString(key) }
        if (kind == null || decision == null) null else kind to decision
    }.toMap()

private fun decoded(json: String): Decisions {
    val sites = JSONObject(json)
    return Decisions(sites.keys().asSequence().associate { Origin(it) to decodedSite(sites.getJSONObject(it)) })
}

data class Decisions(private val sites: Map<Origin, Map<Kind, Decision>> = emptyMap()) {
    fun decision(origin: Origin, kinds: Set<Kind>): Decision? {
        val saved = kinds.map { sites[origin]?.get(it) }
        if (Decision.Block in saved) return Decision.Block
        return Decision.Allow.takeIf { saved.all { it == Decision.Allow } }
    }

    fun with(origin: Origin, kinds: Set<Kind>, decision: Decision): Decisions =
        Decisions(sites + (origin to sites[origin].orEmpty() + kinds.associateWith { decision }))

    fun encoded(): String {
        val json = JSONObject()
        sites.forEach { (origin, site) -> json.put(origin.value, JSONObject(site.entries.associate { it.key.name to it.value.name })) }
        return json.toString()
    }

    companion object {
        fun stored(json: String?): Decisions = json?.let(::decoded) ?: Decisions()
    }
}
