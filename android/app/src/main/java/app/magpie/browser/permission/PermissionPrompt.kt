package app.magpie.browser.permission

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import app.magpie.browser.theme.Dialog
import app.magpie.browser.theme.SolidButton
import app.magpie.browser.theme.TextAction
import app.magpie.browser.theme.ToggleEntry

@Composable
fun SitePrompt(request: Request) = key(request) {
    var remembered by remember { mutableStateOf(false) }
    val decide = { decision: Decision -> if (remembered) request.remember(decision) else request.answer(decision) }
    Dialog(
        title = "Allow ${request.origin.address} to ${purpose(request.kinds)}?",
        onDismiss = { request.answer(Decision.Block) },
        confirm = { SolidButton("Allow", onClick = { decide(Decision.Allow) }) },
        cancel = { TextAction("Block", onClick = { decide(Decision.Block) }) },
        body = if (request.rememberable) {
            { ToggleEntry(null, "Remember for this site", checked = remembered, onChange = { remembered = it }) }
        } else {
            null
        },
    )
}

@Composable
fun SystemPrompt(request: SystemRequest) = key(request) {
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { results ->
        if (results.values.all { it }) request.grant() else request.reject()
    }
    LaunchedEffect(Unit) { launcher.launch(request.permissions) }
}
