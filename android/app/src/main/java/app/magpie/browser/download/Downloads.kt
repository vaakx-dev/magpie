package app.magpie.browser.download

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.webkit.URLUtil
import androidx.annotation.RequiresApi
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.core.content.FileProvider
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.mozilla.geckoview.WebResponse
import java.io.File
import java.io.IOException
import java.io.InputStream

enum class Progress { Running, Done, Failed }

class Download(val name: String, val type: String) {
    var progress by mutableStateOf(Progress.Running)
    var uri by mutableStateOf<Uri?>(null)
}

private const val FALLBACK_TYPE = "application/octet-stream"

fun contentUri(context: Context, file: File): Uri = FileProvider.getUriForFile(context, "${context.packageName}.files", file)

class Downloads(private val context: Context, private val scope: CoroutineScope) {
    val list = mutableStateListOf<Download>()

    fun save(response: WebResponse) {
        val body = response.body ?: return
        val type = response.headers["Content-Type"]?.substringBefore(';')?.trim() ?: FALLBACK_TYPE
        val download = Download(URLUtil.guessFileName(response.uri, response.headers["Content-Disposition"], type), type)
        list.add(0, download)
        scope.launch {
            try {
                download.uri = withContext(Dispatchers.IO) { body.use { write(download, it) } }
                download.progress = Progress.Done
            } catch (failure: IOException) {
                download.progress = Progress.Failed
            }
        }
    }

    private fun write(download: Download, body: InputStream): Uri =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) writeShared(download, body) else writePrivate(download, body)

    @RequiresApi(Build.VERSION_CODES.Q)
    private fun writeShared(download: Download, body: InputStream): Uri {
        val resolver = context.contentResolver
        val details = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, download.name)
            put(MediaStore.Downloads.MIME_TYPE, download.type)
            put(MediaStore.Downloads.IS_PENDING, 1)
        }
        val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, details) ?: throw IOException("No download folder")
        resolver.openOutputStream(uri).use { output -> body.copyTo(output ?: throw IOException("Can't write ${download.name}")) }
        resolver.update(uri, ContentValues().apply { put(MediaStore.Downloads.IS_PENDING, 0) }, null, null)
        return uri
    }

    private fun writePrivate(download: Download, body: InputStream): Uri {
        val folder = context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS) ?: throw IOException("No download folder")
        val file = File(folder, download.name)
        file.outputStream().use { body.copyTo(it) }
        return contentUri(context, file)
    }
}
