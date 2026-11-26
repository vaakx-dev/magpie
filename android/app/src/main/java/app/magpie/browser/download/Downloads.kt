package app.magpie.browser.download

import android.content.ActivityNotFoundException
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.text.format.Formatter
import android.webkit.MimeTypeMap
import androidx.annotation.RequiresApi
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.core.content.FileProvider
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.mozilla.geckoview.WebResponse
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import kotlin.coroutines.coroutineContext

enum class Progress { Running, Done, Failed, Cancelled }

class Download(val name: String, val type: String, val total: Long?) {
    var progress by mutableStateOf(Progress.Running)
    var received by mutableLongStateOf(0L)
    var uri by mutableStateOf<Uri?>(null)
    internal var job: Job? = null

    val fraction: Float? get() = total?.takeIf { it > 0 }?.let { (received.toFloat() / it).coerceAtMost(1f) }

    fun cancel() {
        job?.cancel()
    }
}

private class Target(val uri: Uri, val output: OutputStream, val discard: () -> Unit, val publish: () -> Unit = {})

private const val FALLBACK_TYPE = "application/octet-stream"
private val PROGRESS_LABELS = mapOf(
    Progress.Done to "Saved to Downloads",
    Progress.Failed to "Download failed",
    Progress.Cancelled to "Cancelled",
)

fun contentUri(context: Context, file: File): Uri = FileProvider.getUriForFile(context, "${context.packageName}.files", file)

fun Download.detail(context: Context): String {
    PROGRESS_LABELS[progress]?.let { return it }
    val received = Formatter.formatShortFileSize(context, received)
    return total?.let { "$received of ${Formatter.formatShortFileSize(context, it)}" } ?: received
}

fun Download.open(context: Context) {
    val intent = Intent(Intent.ACTION_VIEW).setDataAndType(uri, type).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    try {
        context.startActivity(intent)
    } catch (missing: ActivityNotFoundException) {
        context.startActivity(Intent.createChooser(intent, name))
    }
}

private fun mediaType(response: WebResponse): String =
    response.headers["Content-Type"]?.substringBefore(';')?.trim()?.ifEmpty { null } ?: FALLBACK_TYPE

private fun describe(response: WebResponse): Download {
    val type = mediaType(response)
    val extension = MimeTypeMap.getSingleton().getExtensionFromMimeType(type)
    val name = fileName(response.headers["Content-Disposition"], response.uri, extension)
    return Download(name, type, response.headers["Content-Length"]?.toLongOrNull())
}

class Downloads(private val context: Context, private val scope: CoroutineScope) {
    val list = mutableStateListOf<Download>()
    var announced by mutableStateOf<Download?>(null)
        private set

    fun save(response: WebResponse) {
        val download = describe(response)
        list.add(0, download)
        announced = download
        when (val body = response.body) {
            null -> download.progress = Progress.Failed
            else -> download.job = scope.launch { receive(download, body) }
        }
    }

    fun dismiss(download: Download) {
        if (announced == download) announced = null
    }

    private suspend fun receive(download: Download, body: InputStream) {
        try {
            download.uri = withContext(Dispatchers.IO) { body.use { write(download, it) } }
            download.progress = Progress.Done
        } catch (cancelled: CancellationException) {
            download.progress = Progress.Cancelled
            throw cancelled
        } catch (failure: Exception) {
            download.progress = Progress.Failed
        }
    }

    private suspend fun write(download: Download, body: InputStream): Uri {
        val target = target(download)
        try {
            target.output.use { output -> copy(download, body, output) }
            target.publish()
            return target.uri
        } catch (failure: Exception) {
            target.discard()
            throw failure
        }
    }

    private suspend fun copy(download: Download, body: InputStream, output: OutputStream) {
        val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
        while (true) {
            coroutineContext.ensureActive()
            val read = body.read(buffer)
            if (read < 0) return
            output.write(buffer, 0, read)
            download.received += read
        }
    }

    private fun target(download: Download): Target =
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) sharedTarget(download) else privateTarget(download)

    @RequiresApi(Build.VERSION_CODES.Q)
    private fun sharedTarget(download: Download): Target {
        val resolver = context.contentResolver
        val details = ContentValues().apply {
            put(MediaStore.Downloads.DISPLAY_NAME, download.name)
            put(MediaStore.Downloads.MIME_TYPE, download.type)
            put(MediaStore.Downloads.IS_PENDING, 1)
        }
        val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, details) ?: throw IOException("No download folder")
        val discard = { resolver.delete(uri, null, null); Unit }
        val output = resolver.openOutputStream(uri)
        if (output == null) {
            discard()
            throw IOException("Can't write ${download.name}")
        }
        val publish = { resolver.update(uri, ContentValues().apply { put(MediaStore.Downloads.IS_PENDING, 0) }, null, null); Unit }
        return Target(uri, output, discard, publish)
    }

    private fun privateTarget(download: Download): Target {
        val folder = context.getExternalFilesDir(Environment.DIRECTORY_DOWNLOADS) ?: throw IOException("No download folder")
        val file = File(folder, download.name)
        return Target(contentUri(context, file), file.outputStream(), discard = { file.delete() })
    }
}
