package app.magpie.browser.favorite

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.input.KeyboardType
import app.magpie.browser.theme.Dialog
import app.magpie.browser.theme.Field
import app.magpie.browser.theme.SolidButton
import app.magpie.browser.theme.TextAction

@Composable
fun EditDialog(favorite: Favorite, onSave: (Favorite) -> Unit, onDismiss: () -> Unit) {
    val title = rememberTextFieldState(favorite.title)
    val url = rememberTextFieldState(favorite.url)
    val edited = favorite.edited(title.text.toString(), url.text.toString())
    val save = { edited?.let(onSave) }
    Dialog(
        "Edit bookmark",
        onDismiss,
        confirm = { SolidButton("Save", onClick = { save() }, enabled = edited != null) },
        cancel = { TextAction("Cancel", onDismiss) },
    ) {
        Field(title, "Title", onDone = { save() })
        Field(url, "URL", KeyboardType.Uri, onDone = { save() })
    }
}
