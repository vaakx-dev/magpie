package app.magpie.browser.settings

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts.CreateDocument
import androidx.activity.result.contract.ActivityResultContracts.OpenDocument
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Restore
import androidx.compose.material.icons.outlined.Save
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import app.magpie.browser.backup.BACKUP_TYPE
import app.magpie.browser.backup.BACKUP_TYPES
import app.magpie.browser.backup.Backup
import app.magpie.browser.backup.Parsed
import app.magpie.browser.backup.backupName
import app.magpie.browser.backup.open
import app.magpie.browser.backup.save
import app.magpie.browser.browser.Browser
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.Section
import kotlinx.coroutines.launch

@Composable
fun Backups(browser: Browser) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var message by remember { mutableStateOf<String?>(null) }
    var pending by remember { mutableStateOf<Backup?>(null) }
    val exporter = rememberLauncherForActivityResult(CreateDocument(BACKUP_TYPE)) { uri ->
        uri?.let { scope.launch { message = save(context, it, browser.backup()) } }
    }
    val importer = rememberLauncherForActivityResult(OpenDocument()) { uri ->
        uri?.let {
            scope.launch {
                when (val parsed = open(context, it)) {
                    is Parsed.Valid -> pending = parsed.backup
                    is Parsed.Invalid -> message = parsed.problem
                }
            }
        }
    }
    Section("Backup")
    Card(
        listOf(
            {
                Item(
                    Icons.Outlined.Save,
                    "Export Magpie backup",
                    onClick = { exporter.launch(backupName()) },
                    detail = "Saves spaces, favorites and settings to a file",
                )
            },
            {
                Item(
                    Icons.Outlined.Restore,
                    "Import Magpie backup",
                    onClick = { importer.launch(BACKUP_TYPES) },
                    detail = "Restores spaces, favorites and settings from a file",
                )
            },
        ),
    )
    message?.let { Note(it) }
    pending?.let { backup ->
        Confirm(
            "Restore this backup?",
            "Replaces your settings and favorites with the backup's and adds its spaces. Open tabs stay open.",
            "Restore",
            onConfirm = {
                browser.restore(backup)
                message = "Backup restored"
                pending = null
            },
            onDismiss = { pending = null },
        )
    }
}
