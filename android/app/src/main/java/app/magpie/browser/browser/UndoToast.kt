package app.magpie.browser.browser

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import app.magpie.browser.theme.CONTROL_SHAPE
import kotlinx.coroutines.delay
import kotlin.time.Duration.Companion.seconds

private val LINGER = 4.seconds

@Composable
fun UndoToast(browser: Browser) {
    val closed = browser.closed
    val announced = closed.announced ?: return
    val colors = MaterialTheme.colorScheme
    LaunchedEffect(announced) {
        delay(LINGER)
        closed.dismiss()
    }
    Row(
        Modifier.clip(CONTROL_SHAPE)
            .background(colors.inverseSurface)
            .clickable(onClick = browser::undo)
            .padding(horizontal = 16.dp, vertical = 10.dp),
    ) {
        Text("Tab closed · ", color = colors.inverseOnSurface, style = MaterialTheme.typography.bodyMedium)
        Text("Undo", color = colors.inverseOnSurface, fontWeight = FontWeight.SemiBold, style = MaterialTheme.typography.bodyMedium)
    }
}
