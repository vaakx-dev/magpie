package app.magpie.browser.favicon

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import app.magpie.browser.browser.Tab
import app.magpie.browser.space.Profile
import app.magpie.browser.theme.host

private const val CORNER_FRACTION = 4
private const val ICON_FRACTION = 2f / 3f
private const val MONOGRAM_FRACTION = 0.4f
private const val NO_LETTER = "·"

val LocalFavicons = staticCompositionLocalOf<Favicons> { error("Favicons are not provided") }

val Tab.retention: Retention get() = if (space.profile == Profile.Private) Retention.Memory else Retention.Disk

private fun monogram(url: String): String = host(url).firstOrNull()?.uppercase() ?: NO_LETTER

@Composable
fun Favicon(url: String, size: Dp, retention: Retention = Retention.Disk) {
    val favicons = LocalFavicons.current
    val origin = remember(url) { Origin.of(url) }
    val icon by produceState(origin?.let(favicons::cached), origin, favicons.revision) {
        value = origin?.let(favicons::cached)
        value = origin?.let { favicons.icon(it, retention) }
    }
    Box(
        Modifier.size(size).clip(RoundedCornerShape(size / CORNER_FRACTION)).background(MaterialTheme.colorScheme.surfaceVariant),
        contentAlignment = Alignment.Center,
    ) {
        when (val loaded = icon) {
            null -> Text(
                monogram(url),
                fontSize = with(LocalDensity.current) { (size * MONOGRAM_FRACTION).toSp() },
                fontWeight = FontWeight.SemiBold,
            )
            else -> Image(loaded, null, Modifier.size(size * ICON_FRACTION), filterQuality = FilterQuality.Medium)
        }
    }
}
