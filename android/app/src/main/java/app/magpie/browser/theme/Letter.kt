package app.magpie.browser.theme

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import java.net.URI

private val SCHEME_PREFIX = Regex("^[a-z][a-z0-9+.-]*://", RegexOption.IGNORE_CASE)

fun host(url: String): String =
    runCatching { URI(url).host }.getOrNull()?.removePrefix("www.") ?: url.replace(SCHEME_PREFIX, "")

@Composable
fun Letter(url: String, size: Dp = 32.dp) {
    Box(
        Modifier.size(size).clip(RoundedCornerShape(size / 4)).background(MaterialTheme.colorScheme.surfaceVariant),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            host(url).firstOrNull()?.uppercase() ?: "·",
            style = MaterialTheme.typography.labelLarge,
            fontWeight = FontWeight.SemiBold,
        )
    }
}
