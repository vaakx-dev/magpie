package app.magpie.browser.settings

import java.net.URI
import java.net.URLEncoder

private const val QUERY = "%s"
private const val FALLBACK_TEMPLATE = "https://duckduckgo.com/?q=%s"
private val WEB_SCHEMES = setOf("http", "https")

enum class SearchEngine(val label: String, val template: String?) {
    DuckDuckGo("DuckDuckGo", FALLBACK_TEMPLATE),
    Google("Google", "https://www.google.com/search?q=%s"),
    Brave("Brave Search", "https://search.brave.com/search?q=%s"),
    Bing("Bing", "https://www.bing.com/search?q=%s"),
    Kagi("Kagi", "https://kagi.com/search?q=%s"),
    Custom("Custom", null),
}

private fun filled(template: String, query: String): String =
    template.replace(QUERY, URLEncoder.encode(query, Charsets.UTF_8.name()))

private fun web(url: String): Boolean {
    val uri = runCatching { URI(url) }.getOrNull() ?: return false
    return uri.scheme?.lowercase() in WEB_SCHEMES && !uri.host.isNullOrEmpty()
}

private fun valid(template: String): Boolean = QUERY in template && web(filled(template, ""))

data class Search(val engine: SearchEngine = SearchEngine.DuckDuckGo, val custom: String = "") {
    val customValid: Boolean get() = valid(custom)

    fun url(query: String): String {
        val template = engine.template ?: custom.takeIf(::valid) ?: FALLBACK_TEMPLATE
        return filled(template, query)
    }
}
