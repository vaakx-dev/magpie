package app.magpie.browser.address

import app.magpie.browser.settings.SearchEngine

private val SCHEME = Regex("^(https?|about|file|data|view-source|resource|moz-extension):", RegexOption.IGNORE_CASE)
private val HOST = Regex(
    "^(localhost|\\d{1,3}(\\.\\d{1,3}){3}|([\\w-]+\\.)+[a-z]{2,}|\\[[0-9a-f:]+])(:\\d+)?([/?#]\\S*)?$",
    RegexOption.IGNORE_CASE,
)

fun looksLikeAddress(text: String): Boolean = SCHEME.containsMatchIn(text) || HOST.matches(text)

fun resolve(input: String, engine: SearchEngine): String {
    val text = input.trim()
    if (SCHEME.containsMatchIn(text)) return text
    if (HOST.matches(text)) return "https://$text"
    return engine.search(text)
}
