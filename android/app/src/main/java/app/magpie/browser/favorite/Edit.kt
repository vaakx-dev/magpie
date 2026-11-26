package app.magpie.browser.favorite

import java.net.URI

private val SCHEME = Regex("^[a-z][a-z0-9+.-]*://", RegexOption.IGNORE_CASE)
private const val DEFAULT_SCHEME = "https://"

private fun withScheme(url: String): String = if (SCHEME.containsMatchIn(url)) url else DEFAULT_SCHEME + url

private fun hasHost(url: String): Boolean = runCatching { URI(url).host }.getOrNull()?.isNotBlank() == true

private fun normalized(url: String): String? = withScheme(url.trim()).takeIf(::hasHost)

fun Favorite.edited(title: String, url: String): Favorite? =
    normalized(url)?.let { Favorite(it, title.trim().ifBlank { this.title }) }
