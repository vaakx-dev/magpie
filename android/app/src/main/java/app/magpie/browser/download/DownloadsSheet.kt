package app.magpie.browser.download

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Download
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import app.magpie.browser.theme.Entry
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.Title

private val PROGRESS_LABELS = mapOf(
    Progress.Running to "Downloading…",
    Progress.Done to "Saved to Downloads",
    Progress.Failed to "Failed",
)

private fun Download.view(context: Context) {
    val intent = Intent(Intent.ACTION_VIEW).setDataAndType(uri, type).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    try {
        context.startActivity(intent)
    } catch (missing: ActivityNotFoundException) {
        context.startActivity(Intent.createChooser(intent, name))
    }
}

@Composable
fun DownloadsSheet(downloads: Downloads, onDismiss: () -> Unit) {
    val context = LocalContext.current
    Sheet(onDismiss) {
        Title("Downloads")
        if (downloads.list.isEmpty()) Note("Files you download show up here.")
        LazyColumn(Modifier.heightIn(max = 480.dp)) {
            items(downloads.list) { download ->
                Entry(
                    Icons.Outlined.Download,
                    download.name,
                    detail = PROGRESS_LABELS.getValue(download.progress),
                    enabled = download.progress == Progress.Done,
                    onClick = { download.view(context) },
                )
            }
        }
    }
}
