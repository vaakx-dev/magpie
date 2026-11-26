package app.magpie.browser.address

import android.content.Intent

private val WEB = Regex("^https?://", RegexOption.IGNORE_CASE)

fun link(action: String?, data: String?): String? =
    data?.takeIf { action == Intent.ACTION_VIEW && WEB.containsMatchIn(it) }
