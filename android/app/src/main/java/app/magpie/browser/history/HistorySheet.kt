package app.magpie.browser.history

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.SiteEntry
import app.magpie.browser.theme.Title

@Composable
fun HistorySheet(browser: Browser, onDismiss: () -> Unit) {
    val history = browser.history
    Sheet(onDismiss) {
        Row(
            Modifier.fillMaxWidth().padding(end = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Title("History")
            TextButton(enabled = history.visits.isNotEmpty(), onClick = history::clear) { Text("Clear") }
        }
        if (history.visits.isEmpty()) Note("Pages you visit show up here. Private tabs are never saved.")
        LazyColumn(Modifier.heightIn(max = 480.dp)) {
            items(history.visits, key = { it.url }) { visit ->
                SiteEntry(visit.url, visit.title.ifBlank { visit.url }, onClick = {
                    browser.load(visit.url)
                    onDismiss()
                })
            }
        }
    }
}
