package app.magpie.browser.browser

import android.graphics.Bitmap
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import kotlinx.coroutines.suspendCancellableCoroutine
import org.mozilla.geckoview.GeckoView
import kotlin.coroutines.resume

private const val THUMBNAIL_WIDTH_PX = 480

private fun thumbnail(page: Bitmap): ImageBitmap {
    val scale = THUMBNAIL_WIDTH_PX.toFloat() / page.width
    return Bitmap.createScaledBitmap(page, THUMBNAIL_WIDTH_PX, (page.height * scale).toInt(), true).asImageBitmap()
}

private suspend fun GeckoView.pixels(): Bitmap? = suspendCancellableCoroutine { continuation ->
    capturePixels().accept({ continuation.resume(it) }, { continuation.resume(null) })
}

suspend fun snapshot(view: GeckoView): ImageBitmap? = view.pixels()?.let(::thumbnail)
