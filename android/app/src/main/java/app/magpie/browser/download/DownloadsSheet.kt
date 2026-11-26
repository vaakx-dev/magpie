package app.magpie.browser.download

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
                    detail = download.detail(context),
                    enabled = download.progress == Progress.Done,
                    onClick = { download.open(context) },
                )
            }
        }
    }
}
