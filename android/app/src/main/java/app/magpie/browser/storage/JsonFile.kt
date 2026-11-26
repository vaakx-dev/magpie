package app.magpie.browser.storage

import android.content.Context
import androidx.core.util.AtomicFile
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONException
import org.json.JSONObject
import java.io.File
import java.io.FileNotFoundException
import java.io.IOException

private const val CORRUPT_SUFFIX = ".corrupt"

class JsonFile(private val path: File, private val scope: CoroutineScope) {
    constructor(context: Context, name: String, scope: CoroutineScope) : this(File(context.filesDir, name), scope)

    private val file = AtomicFile(path)
    private val writes = Dispatchers.IO.limitedParallelism(1)
    private val lock = Any()
    private var pending: String? = null

    fun <T> read(decode: (JSONObject) -> T): T? {
        val text = try {
            file.readFully().decodeToString()
        } catch (missing: FileNotFoundException) {
            return null
        }
        return try {
            decode(JSONObject(text))
        } catch (corrupt: JSONException) {
            setAside()
            null
        }
    }

    fun write(json: JSONObject) {
        val text = json.toString()
        synchronized(lock) { pending = text }
        scope.launch(writes) { flush() }
    }

    fun flush(): Boolean = synchronized(lock) {
        val text = pending ?: return true
        val saved = save(text)
        if (saved) pending = null
        saved
    }

    private fun save(text: String): Boolean {
        val stream = try {
            file.startWrite()
        } catch (unwritable: IOException) {
            return false
        }
        return try {
            stream.write(text.encodeToByteArray())
            file.finishWrite(stream)
            true
        } catch (failed: IOException) {
            file.failWrite(stream)
            false
        }
    }

    private fun setAside() {
        val copy = File(path.path + CORRUPT_SUFFIX)
        copy.delete()
        path.renameTo(copy)
    }
}
