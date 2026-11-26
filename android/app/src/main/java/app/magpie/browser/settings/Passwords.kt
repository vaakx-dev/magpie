package app.magpie.browser.settings

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import app.magpie.browser.browser.Browser
import app.magpie.browser.password.Lock
import app.magpie.browser.password.Status
import app.magpie.browser.password.summary
import kotlinx.coroutines.launch

@Composable
fun Passwords(browser: Browser, onOpen: () -> Unit) {
    val vault = browser.vault
    val scope = rememberCoroutineScope()
    var status by remember { mutableStateOf<Status?>(null) }
    LaunchedEffect(Unit) { status = vault.status() }
    val lines = buildList<Line> {
        add { Item(Icons.Outlined.Key, "Bitwarden", onClick = onOpen, detail = summary(status)) }
        if (status?.lock != Lock.Unavailable) add { Link(Icons.AutoMirrored.Outlined.OpenInNew, "Open Bitwarden", onOpen) }
        if (status?.lock == Lock.Unlocked) add { Item(Icons.Outlined.Lock, "Lock", onClick = { scope.launch { status = vault.lock() } }) }
    }
    Card(lines)
}
