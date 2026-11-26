package app.magpie.browser.permission

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.selection.toggleable
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import app.magpie.browser.prompt.Cancel
import app.magpie.browser.prompt.Confirm
import app.magpie.browser.prompt.Dialog

@Composable
private fun Remember(checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(
        Modifier.fillMaxWidth().toggleable(checked, role = Role.Checkbox, onValueChange = onChange),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Checkbox(checked, onCheckedChange = null)
        Text("Remember for this site", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurface)
    }
}

@Composable
fun SitePrompt(request: Request) = key(request) {
    var remembered by remember { mutableStateOf(false) }
    val decide = { decision: Decision -> if (remembered) request.remember(decision) else request.answer(decision) }
    Dialog(
        title = "Allow ${request.origin.address} to ${purpose(request.kinds)}?",
        onDismiss = { request.answer(Decision.Block) },
        confirm = { Confirm("Allow") { decide(Decision.Allow) } },
        cancel = { Cancel("Block") { decide(Decision.Block) } },
    ) {
        if (request.rememberable) Remember(remembered) { remembered = it }
    }
}

@Composable
fun SystemPrompt(request: SystemRequest) = key(request) {
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { results ->
        if (results.values.all { it }) request.grant() else request.reject()
    }
    LaunchedEffect(Unit) { launcher.launch(request.permissions) }
}
