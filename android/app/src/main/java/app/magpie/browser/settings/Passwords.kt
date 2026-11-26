package app.magpie.browser.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Password
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser
import app.magpie.browser.password.Service
import app.magpie.browser.password.Status
import app.magpie.browser.password.changeAutofillService
import app.magpie.browser.password.summary
import app.magpie.browser.theme.Note

@Composable
private fun ServiceOption(service: Service, picked: Boolean, onPick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier.fillMaxWidth().clickable(onClick = onPick).padding(start = 4.dp, end = 16.dp, top = 12.dp, bottom = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        RadioButton(selected = picked, onClick = onPick)
        Column(Modifier.weight(1f)) {
            Text(service.overline, color = colors.onSurfaceVariant, style = MaterialTheme.typography.labelSmall)
            Text(service.label, color = colors.onSurface, style = MaterialTheme.typography.bodyLarge)
            Text(service.detail, color = colors.onSurfaceVariant, style = MaterialTheme.typography.bodySmall)
        }
    }
}

@Composable
private fun SystemAutofill() {
    val context = LocalContext.current
    Card(listOf { Link(Icons.Outlined.Password, "Change autofill service", onClick = { changeAutofillService(context) }) })
}

@Composable
private fun BuiltInVault(browser: Browser, onOpen: () -> Unit) {
    var status by remember { mutableStateOf<Status?>(null) }
    LaunchedEffect(Unit) { status = browser.vault.status() }
    Card(listOf { Link(Icons.Outlined.Key, "Open vault", onClick = onOpen, detail = summary(status)) })
}

@Composable
fun Passwords(browser: Browser, onOpen: () -> Unit) {
    val settings = browser.settings
    Note("Pick where logins live. Magpie fills from that one place.")
    Card(Service.entries.map { service -> { ServiceOption(service, settings.passwordService == service) { settings.passwordService = service } } })
    when (settings.passwordService) {
        Service.SystemAutofill -> SystemAutofill()
        Service.BuiltIn -> BuiltInVault(browser, onOpen)
    }
}
