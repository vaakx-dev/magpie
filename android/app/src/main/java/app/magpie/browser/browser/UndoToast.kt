package app.magpie.browser.browser

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlin.time.Duration.Companion.seconds

private val SHAPE = RoundedCornerShape(50)
private val LINGER = 4.seconds

@Composable
fun UndoToast(browser: Browser, modifier: Modifier = Modifier) {
    val closed = browser.closed
    val announced = closed.announced ?: return
    val colors = MaterialTheme.colorScheme
    LaunchedEffect(announced) {
        delay(LINGER)
        closed.dismiss()
    }
    Box(modifier, contentAlignment = Alignment.BottomCenter) {
        Row(
            Modifier.padding(bottom = 12.dp)
                .clip(SHAPE)
                .background(colors.inverseSurface)
                .clickable(onClick = browser::undo)
                .padding(horizontal = 16.dp, vertical = 8.dp),
        ) {
            Text("Tab closed · ", color = colors.inverseOnSurface, style = MaterialTheme.typography.bodyMedium)
            Text("Undo", color = colors.inverseOnSurface, fontWeight = FontWeight.SemiBold, style = MaterialTheme.typography.bodyMedium)
        }
    }
}
