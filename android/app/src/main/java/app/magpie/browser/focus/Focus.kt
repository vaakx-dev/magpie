package app.magpie.browser.focus

import app.magpie.browser.core.MagpieCore

private const val KEY_SEPARATOR = "\n"

data class Focus(val blocked: Set<Distraction>) {
    private val keys: String get() = blocked.joinToString(KEY_SEPARATOR) { it.key }

    fun blocks(distraction: Distraction): Boolean = distraction in blocked

    fun block(distraction: Distraction): Focus = Focus(blocked + distraction)

    fun allow(distraction: Distraction): Focus = Focus(blocked - distraction)

    fun script(): String? = if (MagpieCore.loaded) MagpieCore.focusScript(keys) else null

    fun update(): String? = if (MagpieCore.loaded) MagpieCore.focusUpdate(keys) else null

    fun redirect(url: String): String? = if (MagpieCore.loaded) MagpieCore.focusRedirect(keys, url) else null
}
