package app.magpie.browser.prompt

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

private val SHAPE = RoundedCornerShape(12.dp)
private val CONTROL_SHAPE = RoundedCornerShape(8.dp)
private val BODY_MAX_HEIGHT = 320.dp

@Composable
fun Confirm(label: String, enabled: Boolean = true, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Button(
        onClick = onClick,
        enabled = enabled,
        shape = CONTROL_SHAPE,
        colors = ButtonDefaults.buttonColors(containerColor = colors.onSurface, contentColor = colors.surface),
        elevation = null,
    ) { Text(label) }
}

@Composable
fun Cancel(label: String, onClick: () -> Unit) {
    TextButton(onClick = onClick, shape = CONTROL_SHAPE) { Text(label, color = MaterialTheme.colorScheme.onSurface) }
}

@Composable
fun Message(text: String) {
    Text(text, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurface)
}

@Composable
fun Dialog(
    title: String,
    onDismiss: () -> Unit,
    confirm: @Composable () -> Unit,
    cancel: @Composable () -> Unit = {},
    body: @Composable ColumnScope.() -> Unit = {},
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        shape = SHAPE,
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = 0.dp,
        title = { Text(title, style = MaterialTheme.typography.titleMedium) },
        text = { Column(Modifier.heightIn(max = BODY_MAX_HEIGHT).verticalScroll(rememberScrollState()), content = body) },
        confirmButton = confirm,
        dismissButton = cancel,
    )
}
