package app.magpie.browser.theme

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

private val BODY_MAX_HEIGHT = 320.dp
private val SPACING = 12.dp

@Composable
fun Message(text: String) {
    Text(
        text,
        modifier = Modifier.padding(horizontal = GUTTER),
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurface,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Dialog(
    title: String,
    onDismiss: () -> Unit,
    confirm: @Composable () -> Unit,
    cancel: (@Composable () -> Unit)? = null,
    body: (@Composable ColumnScope.() -> Unit)? = null,
) {
    val colors = MaterialTheme.colorScheme
    BasicAlertDialog(onDismissRequest = onDismiss) {
        Surface(shape = CARD_SHAPE, color = colors.surface, contentColor = colors.onSurface) {
            Column(Modifier.padding(vertical = GUTTER), verticalArrangement = Arrangement.spacedBy(SPACING)) {
                Text(title, Modifier.padding(horizontal = GUTTER), style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
                body?.let { Column(Modifier.heightIn(max = BODY_MAX_HEIGHT).verticalScroll(rememberScrollState()), content = it) }
                Row(
                    Modifier.fillMaxWidth().padding(horizontal = GUTTER),
                    horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    cancel?.invoke()
                    confirm()
                }
            }
        }
    }
}

@Composable
fun Confirmation(title: String, text: String, action: String, onConfirm: () -> Unit, onDismiss: () -> Unit) {
    Dialog(
        title,
        onDismiss,
        confirm = { SolidButton(action, onConfirm) },
        cancel = { TextAction("Cancel", onDismiss) },
    ) { Message(text) }
}
