package app.magpie.browser.update

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

private val BANNER_SHAPE = RoundedCornerShape(12.dp)

@Composable
fun UpdateBanner(updates: Updates) {
    if (updates.announced == null) return
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier.padding(horizontal = 12.dp, vertical = 4.dp).fillMaxWidth().clip(BANNER_SHAPE).background(colors.surfaceVariant)
            .padding(start = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            updates.phase.summary(),
            Modifier.weight(1f),
            color = colors.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            style = MaterialTheme.typography.bodyMedium,
        )
        if (updates.installable != null) TextButton(onClick = updates::install) { Text("Install") }
        IconButton(onClick = updates::dismiss) { Icon(Icons.Outlined.Close, "Not now", tint = colors.onSurfaceVariant) }
    }
}
