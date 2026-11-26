package app.magpie.browser.site

import java.net.URI

private val WEB_SCHEMES = setOf("http", "https")
private const val WWW = "www."

@JvmInline
value class Site(val name: String) {
    companion object {
        fun of(url: String): Site? {
            val uri = runCatching { URI(url) }.getOrNull() ?: return null
            if (uri.scheme?.lowercase() !in WEB_SCHEMES) return null
            val host = uri.host?.lowercase() ?: return null
            return Site(host.removePrefix(WWW))
        }
    }
}
