package app.magpie.browser.backup

import android.content.Context
import android.net.Uri
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.IOException
import java.time.LocalDate

const val BACKUP_TYPE = "application/json"
val BACKUP_TYPES = arrayOf(BACKUP_TYPE, "text/plain", "application/octet-stream")

private const val TRUNCATE = "wt"

fun backupName(): String = "magpie-backup-${LocalDate.now()}.json"

suspend fun save(context: Context, uri: Uri, backup: Backup): String = withContext(Dispatchers.IO) {
    try {
        val stream = context.contentResolver.openOutputStream(uri, TRUNCATE) ?: return@withContext "Couldn't open that file."
        stream.bufferedWriter().use { it.write(backup.json()) }
        "Backup saved"
    } catch (failed: IOException) {
        "Couldn't save the backup: ${failed.message}"
    } catch (denied: SecurityException) {
        "Couldn't save the backup: ${denied.message}"
    }
}

suspend fun open(context: Context, uri: Uri): Parsed = withContext(Dispatchers.IO) {
    try {
        val stream = context.contentResolver.openInputStream(uri) ?: return@withContext Parsed.Invalid("Couldn't open that file.")
        parse(stream.bufferedReader().use { it.readText() })
    } catch (failed: IOException) {
        Parsed.Invalid("Couldn't read the backup: ${failed.message}")
    } catch (denied: SecurityException) {
        Parsed.Invalid("Couldn't read the backup: ${denied.message}")
    }
}
