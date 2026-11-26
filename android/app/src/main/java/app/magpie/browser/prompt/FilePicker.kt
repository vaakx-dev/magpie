package app.magpie.browser.prompt

import android.content.Context
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.LocalContext
import kotlinx.coroutines.launch
import org.mozilla.geckoview.GeckoSession.PromptDelegate.FilePrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.PromptResponse
import java.io.IOException

private const val ANY_TYPE = "*/*"

fun documentTypes(mimeTypes: Array<String>?): Array<String> {
    val types = mimeTypes.orEmpty().map { it.trim().lowercase() }.filter { '/' in it }.distinct()
    return types.ifEmpty { listOf(ANY_TYPE) }.toTypedArray()
}

private suspend fun response(context: Context, request: FilePrompt, picked: List<Uri>): PromptResponse {
    if (picked.isEmpty()) return request.dismiss()
    val files = try {
        uploads(context, picked)
    } catch (unreadable: IOException) {
        return request.dismiss()
    } catch (forbidden: SecurityException) {
        return request.dismiss()
    }
    return request.confirm(context, files.toTypedArray())
}

@Composable
private fun SingleFile(types: Array<String>, onPick: (List<Uri>) -> Unit) {
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> onPick(listOfNotNull(uri)) }
    LaunchedEffect(Unit) { launcher.launch(types) }
}

@Composable
private fun MultipleFiles(types: Array<String>, onPick: (List<Uri>) -> Unit) {
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments(), onPick)
    LaunchedEffect(Unit) { launcher.launch(types) }
}

@Composable
fun FilePicker(prompt: Prompt, request: FilePrompt) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val types = documentTypes(request.mimeTypes)
    val deliver: (List<Uri>) -> Unit = { picked -> scope.launch { prompt.answer(response(context, request, picked)) } }
    if (request.type == FilePrompt.Type.MULTIPLE) MultipleFiles(types, deliver) else SingleFile(types, deliver)
}
