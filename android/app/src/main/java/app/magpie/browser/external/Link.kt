package app.magpie.browser.external

import java.net.URLDecoder

private val SCHEME = Regex("^([a-z][a-z0-9+.-]*):", RegexOption.IGNORE_CASE)
private val WEB = Regex("^https?://", RegexOption.IGNORE_CASE)
private val ENGINE_SCHEMES = setOf(
    "http", "https", "about", "data", "blob", "javascript", "file", "content", "resource", "moz-extension", "view-source",
)
private const val INTENT_SCHEME = "intent"
private const val INTENT_FRAGMENT = "#Intent;"
private const val FALLBACK = "S.browser_fallback_url="

private fun scheme(uri: String): String? = SCHEME.find(uri)?.groupValues?.get(1)?.lowercase()

fun leavesBrowser(uri: String): Boolean = scheme(uri)?.let { it !in ENGINE_SCHEMES } == true

fun appIntent(uri: String): Boolean = scheme(uri) == INTENT_SCHEME

fun fallback(uri: String): String? {
    if (!appIntent(uri)) return null
    val extras = uri.substringAfter(INTENT_FRAGMENT, "").split(';')
    val encoded = extras.find { it.startsWith(FALLBACK) }?.removePrefix(FALLBACK) ?: return null
    val decoded = runCatching { URLDecoder.decode(encoded, Charsets.UTF_8.name()) }.getOrNull() ?: return null
    return decoded.takeIf(WEB::containsMatchIn)
}
