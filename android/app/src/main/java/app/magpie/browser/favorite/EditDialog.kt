package app.magpie.browser.favorite

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import app.magpie.browser.theme.Field

private val SHAPE = RoundedCornerShape(12.dp)

@Composable
fun EditDialog(favorite: Favorite, onSave: (Favorite) -> Unit, onDismiss: () -> Unit) {
    val title = rememberTextFieldState(favorite.title)
    val url = rememberTextFieldState(favorite.url)
    val edited = favorite.edited(title.text.toString(), url.text.toString())
    val save = { edited?.let(onSave) }
    AlertDialog(
        onDismissRequest = onDismiss,
        shape = SHAPE,
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = 0.dp,
        title = { Text("Edit bookmark") },
        text = {
            Column {
                Field(title, "Title", onDone = { save() })
                Field(url, "URL", KeyboardType.Uri, onDone = { save() })
            }
        },
        confirmButton = { TextButton(enabled = edited != null, onClick = { save() }) { Text("Save") } },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}
