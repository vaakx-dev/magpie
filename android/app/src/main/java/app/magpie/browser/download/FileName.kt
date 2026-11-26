package app.magpie.browser.download

import java.net.URI
import java.net.URISyntaxException
import java.net.URLDecoder
import java.nio.charset.Charset
import java.nio.charset.IllegalCharsetNameException
import java.nio.charset.UnsupportedCharsetException

private const val UNNAMED = "download"
private val EXTENDED = Regex("""filename\*\s*=\s*([^']*)'[^']*'([^;\s]+)""", RegexOption.IGNORE_CASE)
private val QUOTED = Regex("""filename\s*=\s*"((?:[^"\\]|\\.)*)"""", RegexOption.IGNORE_CASE)
private val BARE = Regex("""filename\s*=\s*([^;\s]+)""", RegexOption.IGNORE_CASE)
private val ESCAPE = Regex("""\\(.)""")
private val UNSAFE = Regex("""[\\/:*?"<>|\p{Cntrl}]""")

private fun decoded(encoded: String, charset: Charset): String? =
    runCatching { URLDecoder.decode(encoded.replace("+", "%2B"), charset.name()) }.getOrNull()

private fun charset(name: String): Charset =
    try {
        Charset.forName(name.ifBlank { Charsets.UTF_8.name() })
    } catch (unknown: IllegalCharsetNameException) {
        Charsets.UTF_8
    } catch (unknown: UnsupportedCharsetException) {
        Charsets.UTF_8
    }

private fun extended(disposition: String): String? {
    val (charsetName, encoded) = EXTENDED.find(disposition)?.destructured ?: return null
    return decoded(encoded, charset(charsetName))
}

private fun quoted(disposition: String): String? =
    QUOTED.find(disposition)?.groupValues?.get(1)?.replace(ESCAPE, "$1")

private fun bare(disposition: String): String? = BARE.find(disposition)?.groupValues?.get(1)

private fun fromDisposition(disposition: String): String? =
    extended(disposition) ?: quoted(disposition) ?: bare(disposition)

private fun fromUrl(url: String): String? {
    val path = try {
        URI(url).rawPath
    } catch (malformed: URISyntaxException) {
        null
    }
    val segment = path?.substringAfterLast('/')?.takeIf { it.isNotEmpty() } ?: return null
    return decoded(segment, Charsets.UTF_8)
}

private fun safe(name: String): String? = name.replace(UNSAFE, "_").trim(' ', '.').takeIf { it.isNotEmpty() }

private fun withExtension(name: String, extension: String?): String =
    if (extension == null || name.lastIndexOf('.') > 0) name else "$name.$extension"

fun fileName(disposition: String?, url: String, extension: String?): String {
    val named = disposition?.let(::fromDisposition)?.let(::safe) ?: fromUrl(url)?.let(::safe) ?: UNNAMED
    return withExtension(named, extension)
}
