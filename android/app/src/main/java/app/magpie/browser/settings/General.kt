package app.magpie.browser.settings

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.CleaningServices
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Edit
import androidx.compose.material.icons.outlined.Layers
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.SwipeLeft
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
import app.magpie.browser.browser.Browser
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import app.magpie.browser.space.SpaceDialog
import app.magpie.browser.theme.Field
import app.magpie.browser.theme.Section

@Composable
private fun RenameDialog(space: Space, onRename: (String) -> Unit, onDismiss: () -> Unit) {
    val name = rememberTextFieldState(space.name)
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = 0.dp,
        title = { Text("Rename ${space.name}") },
        text = { Field(name, "Name") },
        confirmButton = {
            TextButton(enabled = name.text.isNotBlank(), onClick = { onRename(name.text.trim().toString()) }) { Text("Rename") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}

@Composable
private fun SpaceLine(browser: Browser, space: Space, onRename: () -> Unit) {
    val current = if (space == browser.space) " · Current" else ""
    Item(Icons.Outlined.Layers, space.name, onClick = { browser.switch(space) }, detail = "Profile: ${space.profile.label}$current") {
        ActionIcon(Icons.Outlined.Edit, "Rename ${space.name}", onRename)
        if (browser.spaces.saved.size > 1) ActionIcon(Icons.Outlined.Delete, "Delete ${space.name}") { browser.removeSpace(space) }
    }
}

@Composable
fun General(browser: Browser) {
    var creating by remember { mutableStateOf(false) }
    var renaming by remember { mutableStateOf<Space?>(null) }
    var clearing by remember { mutableStateOf<Profile.Named?>(null) }
    val spaces = browser.spaces
    val settings = browser.settings
    Section("Browsing")
    Card(
        listOf {
            SwitchItem(
                Icons.Outlined.SwipeLeft,
                "Swipe address bar to switch tabs",
                checked = settings.swipeTabs,
                onChange = { settings.swipeTabs = it },
                detail = "Swipe up on it to see all tabs",
            )
        },
    )
    Section("Spaces")
    Card(
        spaces.saved.map<Space, Line> { space -> { SpaceLine(browser, space) { renaming = space } } } +
            { Item(Icons.Outlined.Add, "New space", onClick = { creating = true }) },
    )
    Section("Profiles")
    Card(
        spaces.profiles.map { profile ->
            {
                val users = spaces.saved.filter { it.profile == profile }.joinToString { it.name }
                Item(Icons.Outlined.Person, profile.label, onClick = { clearing = profile }, detail = "Used by $users") {
                    ActionIcon(Icons.Outlined.CleaningServices, "Clear ${profile.label} data") { clearing = profile }
                }
            }
        },
    )
    Backups(browser)
    if (creating) {
        SpaceDialog(
            spaces.profiles,
            onCreate = { name, profile ->
                browser.addSpace(name, profile)
                creating = false
            },
            onDismiss = { creating = false },
        )
    }
    renaming?.let { space ->
        RenameDialog(
            space,
            onRename = { name ->
                browser.renameSpace(space, name)
                renaming = null
            },
            onDismiss = { renaming = null },
        )
    }
    clearing?.let { profile ->
        ConfirmClear(
            "Clear ${profile.label}?",
            "Signs you out of every site in this profile and deletes its cookies, storage and cache.",
            onConfirm = {
                browser.engine.clear(profile)
                clearing = null
            },
            onDismiss = { clearing = null },
        )
    }
}
