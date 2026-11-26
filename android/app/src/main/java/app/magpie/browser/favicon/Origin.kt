package app.magpie.browser.favicon

import java.net.URI

private val WEB_SCHEMES = setOf("http", "https")
private val UNSAFE_FILE_CHARACTERS = Regex("[^a-z0-9.-]")
private const val NO_PORT = -1

@JvmInline
value class Origin(val value: String) {
    val fileName: String get() = value.replace(UNSAFE_FILE_CHARACTERS, "_") + ".png"

    val address: String get() = value.substringAfter("://")

    val guesses: List<String> get() = listOf("$value/apple-touch-icon.png", "$value/favicon.ico")

    companion object {
        fun of(url: String): Origin? {
            val uri = runCatching { URI(url) }.getOrNull() ?: return null
            val scheme = uri.scheme?.lowercase()?.takeIf { it in WEB_SCHEMES } ?: return null
            val host = uri.host?.lowercase() ?: return null
            val port = if (uri.port == NO_PORT) "" else ":${uri.port}"
            return Origin("$scheme://$host$port")
        }
    }
}
