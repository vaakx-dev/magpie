package app.magpie.browser.favicon

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.util.LruCache
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.core.graphics.scale
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL

enum class Retention { Disk, Memory }

private const val FOLDER = "favicons"
private const val MEMORY_ICONS = 128
private const val TIMEOUT_MS = 5_000
private const val MAX_BYTES = 512 * 1024
private const val MAX_PIXELS = 128
private const val MAX_HINTS = 4
private const val PNG_QUALITY = 100

private fun InputStream.readAtMost(limit: Int): ByteArray? {
    val bytes = ByteArrayOutputStream()
    val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
    while (true) {
        val read = read(buffer)
        if (read < 0) return bytes.toByteArray()
        bytes.write(buffer, 0, read)
        if (bytes.size() > limit) return null
    }
}

private fun HttpURLConnection.body(): ByteArray? {
    connectTimeout = TIMEOUT_MS
    readTimeout = TIMEOUT_MS
    if (responseCode != HttpURLConnection.HTTP_OK) return null
    return inputStream.use { it.readAtMost(MAX_BYTES) }
}

private fun download(url: String): ByteArray? {
    val connection = try {
        URL(url).openConnection() as? HttpURLConnection
    } catch (_: IOException) {
        null
    } ?: return null
    return try {
        connection.body()
    } catch (_: IOException) {
        null
    } finally {
        connection.disconnect()
    }
}

private fun Bitmap.capped(): Bitmap {
    val largest = maxOf(width, height)
    if (largest <= MAX_PIXELS) return this
    return scale(width * MAX_PIXELS / largest, height * MAX_PIXELS / largest)
}

private fun decode(bytes: ByteArray): Bitmap? {
    val bitmap = when (val ico = decodeIco(bytes)) {
        is IcoImage.Png -> BitmapFactory.decodeByteArray(ico.bytes, 0, ico.bytes.size)
        is IcoImage.Pixels -> Bitmap.createBitmap(ico.argb, ico.width, ico.height, Bitmap.Config.ARGB_8888)
        null -> BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
    }
    return bitmap?.capped()
}

class Favicons(context: Context, private val scope: CoroutineScope) {
    private val folder = File(context.cacheDir, FOLDER)
    private val memory = LruCache<Origin, ImageBitmap>(MEMORY_ICONS)
    private val missing = mutableSetOf<Origin>()
    private val learned = mutableSetOf<Origin>()
    private val loading = mutableMapOf<Origin, Deferred<ImageBitmap?>>()
    var revision by mutableIntStateOf(0)
        private set

    fun cached(origin: Origin): ImageBitmap? = memory[origin]

    suspend fun icon(origin: Origin, retention: Retention): ImageBitmap? {
        memory[origin]?.let { return it }
        if (origin in missing) return null
        return loading.getOrPut(origin) { scope.async { settle(origin, find(origin, retention)) } }.await()
    }

    fun learn(page: String, icons: List<String>, retention: Retention) {
        val origin = Origin.of(page) ?: return
        if (!learned.add(origin)) return
        val hints = icons.filter { Origin.of(it) != null }.take(MAX_HINTS)
        scope.launch {
            val icon = fetch(origin, hints, retention) ?: return@launch
            memory.put(origin, icon)
            missing.remove(origin)
            revision++
        }
    }

    fun clear() {
        memory.evictAll()
        missing.clear()
        learned.clear()
        revision++
        scope.launch(Dispatchers.IO) { folder.deleteRecursively() }
    }

    private fun settle(origin: Origin, found: ImageBitmap?): ImageBitmap? {
        loading.remove(origin)
        val known = memory[origin] ?: found
        if (known == null) missing.add(origin) else memory.put(origin, known)
        return known
    }

    private suspend fun find(origin: Origin, retention: Retention): ImageBitmap? =
        withContext(Dispatchers.IO) { read(origin)?.asImageBitmap() } ?: fetch(origin, origin.guesses, retention)

    private suspend fun fetch(origin: Origin, urls: List<String>, retention: Retention): ImageBitmap? = withContext(Dispatchers.IO) {
        val bitmap = urls.firstNotNullOfOrNull { url -> download(url)?.let(::decode) } ?: return@withContext null
        if (retention == Retention.Disk) write(origin, bitmap)
        bitmap.asImageBitmap()
    }

    private fun file(origin: Origin): File = File(folder, origin.fileName)

    private fun read(origin: Origin): Bitmap? = BitmapFactory.decodeFile(file(origin).path)

    private fun write(origin: Origin, bitmap: Bitmap) {
        folder.mkdirs()
        val partial = File(folder, "${origin.fileName}.part")
        partial.outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, PNG_QUALITY, it) }
        partial.renameTo(file(origin))
    }
}
