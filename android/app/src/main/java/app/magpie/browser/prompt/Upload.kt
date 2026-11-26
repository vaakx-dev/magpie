package app.magpie.browser.prompt

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileNotFoundException
import java.util.UUID

private const val UPLOADS = "uploads"
private const val UNNAMED = "upload"
private val SEPARATORS = Regex("""[\\/]""")

private fun displayName(context: Context, uri: Uri): String? =
    context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
        if (cursor.moveToFirst()) cursor.getString(0) else null
    }

private fun safeName(name: String?): String =
    name?.replace(SEPARATORS, "_")?.takeIf { it.trim('.', ' ').isNotEmpty() } ?: UNNAMED

private fun copy(context: Context, uri: Uri): Uri {
    val folder = File(context.cacheDir, "$UPLOADS/${UUID.randomUUID()}").apply { mkdirs() }
    val file = File(folder, safeName(displayName(context, uri)))
    val input = context.contentResolver.openInputStream(uri) ?: throw FileNotFoundException(uri.toString())
    input.use { source -> file.outputStream().use { source.copyTo(it) } }
    return Uri.fromFile(file)
}

suspend fun uploads(context: Context, picked: List<Uri>): List<Uri> =
    withContext(Dispatchers.IO) { picked.map { copy(context, it) } }
