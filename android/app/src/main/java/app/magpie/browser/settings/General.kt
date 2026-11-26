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
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import app.magpie.browser.browser.Browser
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import app.magpie.browser.space.SpaceDialog
import app.magpie.browser.theme.Confirmation
import app.magpie.browser.theme.Dialog
import app.magpie.browser.theme.Field
import app.magpie.browser.theme.IconAction
import app.magpie.browser.theme.SolidButton
import app.magpie.browser.theme.TextAction
import app.magpie.browser.theme.Section

@Composable
private fun RenameDialog(space: Space, onRename: (String) -> Unit, onDismiss: () -> Unit) {
    val name = rememberTextFieldState(space.name)
    Dialog(
        "Rename ${space.name}",
        onDismiss,
        confirm = { SolidButton("Rename", onClick = { onRename(name.text.trim().toString()) }, enabled = name.text.isNotBlank()) },
        cancel = { TextAction("Cancel", onDismiss) },
    ) { Field(name, "Name") }
}

@Composable
private fun SpaceLine(browser: Browser, space: Space, onRename: () -> Unit, onDelete: () -> Unit) {
    val current = if (space == browser.space) " · Current" else ""
    Item(Icons.Outlined.Layers, space.name, onClick = { browser.switch(space) }, detail = "Profile: ${space.profile.label}$current") {
        IconAction(Icons.Outlined.Edit, "Rename ${space.name}", onRename)
        if (browser.spaces.saved.size > 1) IconAction(Icons.Outlined.Delete, "Delete ${space.name}", onDelete)
    }
}

@Composable
private fun ConfirmDelete(browser: Browser, space: Space, onDone: () -> Unit) {
    Confirmation(
        "Delete ${space.name}?",
        "Removes this space and closes its tabs. This can't be undone.",
        "Delete",
        onConfirm = {
            browser.removeSpace(space)
            onDone()
        },
        onDismiss = onDone,
    )
}

@Composable
fun General(browser: Browser) {
    var creating by remember { mutableStateOf(false) }
    var renaming by remember { mutableStateOf<Space?>(null) }
    var deleting by remember { mutableStateOf<Space?>(null) }
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
        spaces.saved.map<Space, Line> { space -> { SpaceLine(browser, space, onRename = { renaming = space }, onDelete = { deleting = space }) } } +
            { Item(Icons.Outlined.Add, "New space", onClick = { creating = true }) },
    )
    Section("Profiles")
    Card(
        spaces.profiles.map { profile ->
            {
                val users = spaces.saved.filter { it.profile == profile }.joinToString { it.name }
                Item(Icons.Outlined.Person, profile.label, onClick = { clearing = profile }, detail = "Used by $users") {
                    IconAction(Icons.Outlined.CleaningServices, "Clear ${profile.label} data", onClick = { clearing = profile })
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
    deleting?.let { space -> ConfirmDelete(browser, space, onDone = { deleting = null }) }
    clearing?.let { profile ->
        Confirmation(
            "Clear ${profile.label}?",
            "Signs you out of every site in this profile and deletes its cookies, storage and cache.",
            "Clear",
            onConfirm = {
                browser.engine.clear(profile)
                clearing = null
            },
            onDismiss = { clearing = null },
        )
    }
}
