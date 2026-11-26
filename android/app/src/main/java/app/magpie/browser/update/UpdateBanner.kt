package app.magpie.browser.update

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import app.magpie.browser.theme.IconAction
import app.magpie.browser.theme.Notice
import app.magpie.browser.theme.TextAction

@Composable
fun UpdateBanner(updates: Updates) {
    if (updates.announced == null) return
    Notice {
        Row(Modifier.padding(start = 16.dp, end = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(
                updates.phase.summary(),
                Modifier.weight(1f),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                style = MaterialTheme.typography.bodyMedium,
            )
            if (updates.installable != null) TextAction("Install", updates::install)
            IconAction(Icons.Outlined.Close, "Not now", updates::dismiss)
        }
    }
}
