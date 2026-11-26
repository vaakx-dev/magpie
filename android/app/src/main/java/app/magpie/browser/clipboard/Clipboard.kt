package app.magpie.browser.clipboard

import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import android.os.PersistableBundle

enum class Content { Plain, Secret }

private fun ClipData.markSecret() {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) return
    description.extras = PersistableBundle().apply { putBoolean(ClipDescription.EXTRA_IS_SENSITIVE, true) }
}

fun copy(context: Context, label: String, text: String, content: Content) {
    val clip = ClipData.newPlainText(label, text)
    if (content == Content.Secret) clip.markSecret()
    context.getSystemService(ClipboardManager::class.java).setPrimaryClip(clip)
}
