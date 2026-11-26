package app.magpie.browser.download

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import kotlin.time.Duration.Companion.seconds

private val SHAPE = RoundedCornerShape(12.dp)
private val PROGRESS_HEIGHT = 2.dp
private val LINGER = 6.seconds

@Composable
private fun Bar(download: Download) {
    val colors = MaterialTheme.colorScheme
    val modifier = Modifier.fillMaxWidth().height(PROGRESS_HEIGHT)
    when (val fraction = download.fraction) {
        null -> LinearProgressIndicator(modifier, color = colors.onSurface, trackColor = colors.surfaceVariant, gapSize = 0.dp)
        else -> LinearProgressIndicator(
            progress = { fraction },
            modifier = modifier,
            color = colors.onSurface,
            trackColor = colors.surfaceVariant,
            drawStopIndicator = {},
            gapSize = 0.dp,
        )
    }
}

@Composable
fun DownloadBanner(downloads: Downloads) {
    val download = downloads.announced ?: return
    val context = LocalContext.current
    val colors = MaterialTheme.colorScheme
    val running = download.progress == Progress.Running
    LaunchedEffect(download, running) {
        if (running) return@LaunchedEffect
        delay(LINGER)
        downloads.dismiss(download)
    }
    Column(Modifier.padding(horizontal = 12.dp, vertical = 4.dp).fillMaxWidth().clip(SHAPE).background(colors.surface)) {
        Row(Modifier.padding(start = 14.dp), verticalAlignment = Alignment.CenterVertically) {
            Icon(Icons.Outlined.Download, null, tint = colors.onSurfaceVariant)
            Column(Modifier.weight(1f).padding(start = 12.dp, top = 8.dp, bottom = 8.dp)) {
                Text(download.name, color = colors.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.bodyMedium)
                Text(download.detail(context), color = colors.onSurfaceVariant, maxLines = 1, style = MaterialTheme.typography.bodySmall)
            }
            when (download.progress) {
                Progress.Running -> TextButton(onClick = download::cancel) { Text("Cancel") }
                Progress.Done -> TextButton(onClick = { download.open(context); downloads.dismiss(download) }) { Text("Open") }
                Progress.Failed, Progress.Cancelled -> Unit
            }
            IconButton(onClick = { downloads.dismiss(download) }) { Icon(Icons.Outlined.Close, "Dismiss", tint = colors.onSurfaceVariant) }
        }
        if (running) Bar(download)
    }
}
