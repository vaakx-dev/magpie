package app.magpie.browser.storage

import android.content.Context
import android.util.AtomicFile
import androidx.core.util.readText
import androidx.core.util.writeText
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONException
import org.json.JSONObject
import java.io.File
import java.io.FileNotFoundException

class JsonFile(context: Context, name: String, private val scope: CoroutineScope) {
    private val file = AtomicFile(File(context.filesDir, name))
    private val writes = Dispatchers.IO.limitedParallelism(1)

    fun <T> read(decode: (JSONObject) -> T): T? =
        try {
            decode(JSONObject(file.readText()))
        } catch (missing: FileNotFoundException) {
            null
        } catch (corrupt: JSONException) {
            null
        }

    fun write(json: JSONObject) {
        val text = json.toString()
        scope.launch(writes) { file.writeText(text) }
    }
}
