package app.magpie.browser.space

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.unit.dp
import app.magpie.browser.theme.Field
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.RadioEntry
import app.magpie.browser.theme.Section

@Composable
fun SpaceDialog(profiles: List<Profile.Named>, onCreate: (String, Profile.Named?) -> Unit, onDismiss: () -> Unit) {
    val name = rememberTextFieldState()
    var profile by remember { mutableStateOf<Profile.Named?>(null) }
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = 0.dp,
        title = { Text("New Space") },
        text = {
            Column {
                Field(name, "Name")
                Section("Profile")
                RadioEntry("New profile", selected = profile == null, onSelect = { profile = null }, detail = "Its own cookies and logins")
                profiles.forEach { shared ->
                    RadioEntry(shared.label, selected = profile == shared, onSelect = { profile = shared }, detail = "Shared profile")
                }
                Note("Spaces on the same profile share cookies, storage and logins.")
            }
        },
        confirmButton = {
            TextButton(enabled = name.text.isNotBlank(), onClick = { onCreate(name.text.trim().toString(), profile) }) { Text("Create") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}
