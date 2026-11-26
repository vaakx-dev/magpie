package app.magpie.browser.history

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.SearchBox
import app.magpie.browser.theme.Section
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.SiteEntry
import app.magpie.browser.theme.Title

private val REMOVE_SHAPE = RoundedCornerShape(8.dp)

@Composable
private fun Remove(onClick: () -> Unit) {
    IconButton(onClick = onClick, modifier = Modifier.size(32.dp), shape = REMOVE_SHAPE) {
        Icon(Icons.Outlined.Close, "Remove from history", Modifier.size(17.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
fun HistorySheet(browser: Browser, onDismiss: () -> Unit) {
    val history = browser.history
    var query by rememberSaveable { mutableStateOf("") }
    val groups = grouped(matching(history.visits, query), System.currentTimeMillis())
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
        else SearchBox(query, "Search history", { query = it }, Modifier.padding(start = 12.dp, end = 12.dp, bottom = 4.dp))
        if (history.visits.isNotEmpty() && groups.isEmpty()) Note("No pages match “${query.trim()}”.")
        LazyColumn(Modifier.heightIn(max = 480.dp)) {
            groups.forEach { group ->
                item(key = group.day) { Section(group.day.label) }
                items(group.visits, key = { it.url }) { visit ->
                    SiteEntry(
                        visit.url,
                        visit.title.ifBlank { visit.url },
                        onClick = {
                            browser.load(visit.url)
                            onDismiss()
                        },
                        trailing = { Remove { history.forget(visit.url) } },
                    )
                }
            }
        }
    }
}
