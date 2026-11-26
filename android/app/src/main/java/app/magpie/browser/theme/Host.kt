package app.magpie.browser.theme

import java.net.URI

private val SCHEME_PREFIX = Regex("^[a-z][a-z0-9+.-]*://", RegexOption.IGNORE_CASE)

fun host(url: String): String =
    runCatching { URI(url).host }.getOrNull()?.removePrefix("www.") ?: url.replace(SCHEME_PREFIX, "")
