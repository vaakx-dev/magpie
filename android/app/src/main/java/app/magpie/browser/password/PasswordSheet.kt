package app.magpie.browser.password

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material.icons.outlined.Timer
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser
import app.magpie.browser.clipboard.Content
import app.magpie.browser.clipboard.copy
import app.magpie.browser.theme.Choices
import app.magpie.browser.theme.Entry
import app.magpie.browser.theme.Field
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.SecretField
import app.magpie.browser.theme.Section
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.Title
import kotlinx.coroutines.launch

@Composable
private fun Busy(busy: Boolean) {
    if (!busy) return
    LinearProgressIndicator(
        color = MaterialTheme.colorScheme.onSurface,
        trackColor = Color.Transparent,
        modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp),
    )
}

@Composable
private fun Problem(error: String) {
    if (error.isNotEmpty()) Note(error)
}

@Composable
private fun Action(label: String, enabled: Boolean, onClick: () -> Unit) {
    Button(enabled = enabled, onClick = onClick, modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 8.dp)) {
        Text(label)
    }
}

@Composable
private fun SignIn(status: Status, busy: Boolean, onSignIn: (String, String, String) -> Unit) {
    var server by remember { mutableStateOf(Server.UnitedStates) }
    val selfHosted = rememberTextFieldState()
    val email = rememberTextFieldState(status.email)
    val password = rememberTextFieldState()
    val address = if (server == Server.SelfHosted) selfHosted.text.trim().toString() else server.address
    Section("Server")
    Choices(Server.entries, server, Server::label) { server = it }
    if (server == Server.SelfHosted) Field(selfHosted, "Server address", KeyboardType.Uri)
    Field(email, "Email", KeyboardType.Email)
    SecretField(password, "Master password")
    Problem(status.error)
    Action("Sign in", enabled = !busy && address.isNotEmpty() && email.text.isNotBlank() && password.text.isNotEmpty()) {
        onSignIn(address, email.text.trim().toString(), password.text.toString())
    }
}

private fun hint(method: Method): String = when (method) {
    Method.Authenticator -> "Enter the 6-digit code from your authenticator app."
    Method.Email -> "Enter the code Bitwarden emailed you."
    Method.YubiKey -> "Insert or tap your YubiKey to fill in its code."
    Method.NewDevice -> "Bitwarden emailed you a code to confirm this device."
}

private fun keyboard(method: Method): KeyboardType = if (method == Method.YubiKey) KeyboardType.Password else KeyboardType.Number

@Composable
private fun TwoStep(
    status: Status,
    busy: Boolean,
    emailSent: Boolean,
    onAnswer: (Method, String) -> Unit,
    onSendEmail: () -> Unit,
    onCancel: () -> Unit,
) {
    var method by remember(status.methods) { mutableStateOf(status.methods.first()) }
    val code = rememberTextFieldState()
    Section("Two-step login")
    if (status.methods.size > 1) Choices(status.methods, method, Method::label) { method = it }
    Note(if (method == Method.Email && emailSent) "Code sent. Check your email." else hint(method))
    Field(code, "Verification code", keyboard(method))
    Problem(status.error)
    Action("Verify", enabled = !busy && code.text.isNotBlank()) { onAnswer(method, code.text.trim().toString()) }
    Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp), horizontalArrangement = Arrangement.SpaceBetween) {
        TextButton(enabled = !busy, onClick = onCancel) { Text("Cancel") }
        if (method == Method.Email) TextButton(enabled = !busy, onClick = onSendEmail) { Text("Send email code") }
    }
}

@Composable
private fun Unlock(
    status: Status,
    busy: Boolean,
    onUnlock: (String) -> Unit,
    onSignOut: () -> Unit,
) {
    val password = rememberTextFieldState()
    Note("Locked · ${status.email}")
    SecretField(password, "Master password")
    Problem(status.error)
    Action("Unlock", enabled = !busy && password.text.isNotEmpty()) { onUnlock(password.text.toString()) }
    TextButton(enabled = !busy, onClick = onSignOut, modifier = Modifier.padding(horizontal = 12.dp)) { Text("Sign out") }
}

@Composable
private fun LoginEntry(login: Login, onCopy: (String, String) -> Unit, onCopyCode: () -> Unit) {
    Entry(null, login.name, detail = login.username, onClick = { onCopy("Password", login.password) }) {
        if (login.username.isNotEmpty()) {
            IconButton(onClick = { onCopy("Username", login.username) }) { Icon(Icons.Outlined.Person, "Copy username") }
        }
        IconButton(onClick = { onCopy("Password", login.password) }) { Icon(Icons.Outlined.Key, "Copy password") }
        if (login.totp.isNotEmpty()) {
            IconButton(onClick = onCopyCode) { Icon(Icons.Outlined.Timer, "Copy verification code") }
        }
    }
}

@Composable
private fun Logins(browser: Browser, status: Status, onLock: () -> Unit, onSignOut: () -> Unit) {
    val vault = browser.vault
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val site = browser.tab?.takeUnless { it.blank }?.url.orEmpty()
    val search = rememberTextFieldState()
    val query = search.text.trim().toString()
    var logins by remember { mutableStateOf(emptyList<Login>()) }
    var generated by remember { mutableStateOf("") }
    val copySecret = { label: String, secret: String -> copy(context, label, secret, Content.Secret) }
    LaunchedEffect(query, site) { logins = if (query.isBlank()) vault.matches(site) else vault.search(query) }

    Row(
        Modifier.fillMaxWidth().padding(start = 20.dp, end = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text(status.email, Modifier.weight(1f), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        TextButton(onClick = { scope.launch { generated = vault.generatePassword() } }) { Text("Generate") }
        TextButton(onClick = onLock) { Text("Lock") }
        TextButton(onClick = onSignOut) { Text("Sign out") }
    }
    if (generated.isNotEmpty()) {
        Entry(Icons.Outlined.Key, generated, detail = "Generated password", onClick = { copySecret("Password", generated) }) {
            Icon(Icons.Outlined.ContentCopy, "Copy generated password")
        }
    }
    Field(search, "Search vault")
    Section(if (query.isBlank()) "For this site" else "Results")
    if (logins.isEmpty()) Note(if (query.isBlank()) "No logins for this site." else "No matches.")
    LazyColumn(Modifier.heightIn(max = 360.dp)) {
        items(logins, key = { it.id }) { login ->
            LoginEntry(login, onCopy = copySecret, onCopyCode = { scope.launch { vault.totp(login.totp).takeIf { it.isNotEmpty() }?.let { copySecret("Verification code", it) } } })
        }
    }
}

@Composable
fun PasswordSheet(browser: Browser, onDismiss: () -> Unit) {
    val vault = browser.vault
    val scope = rememberCoroutineScope()
    var status by remember { mutableStateOf<Status?>(null) }
    var busy by remember { mutableStateOf(false) }
    var emailSent by remember { mutableStateOf(false) }
    fun run(call: suspend () -> Status) {
        scope.launch {
            busy = true
            status = call()
            busy = false
        }
    }
    LaunchedEffect(Unit) { status = vault.status() }

    Sheet(onDismiss) {
        Title("Passwords")
        Busy(busy || status == null)
        val current = status ?: return@Sheet
        when (current.lock) {
            Lock.Unavailable -> Note("Bitwarden needs Magpie's core library, which isn't in this build.")
            Lock.SignedOut -> SignIn(current, busy) { server, email, password -> run { vault.signIn(server, email, password) } }
            Lock.Challenged -> TwoStep(
                current,
                busy,
                emailSent,
                onAnswer = { method, code -> run { vault.answer(method, code) } },
                onSendEmail = { run { vault.sendEmailCode().also { emailSent = it.error.isEmpty() } } },
                onCancel = { run { vault.lock() } },
            )
            Lock.Locked -> Unlock(
                current,
                busy,
                onUnlock = { password -> run { vault.unlock(password) } },
                onSignOut = { run { vault.signOut() } },
            )
            Lock.Unlocked -> Logins(browser, current, onLock = { run { vault.lock() } }, onSignOut = { run { vault.signOut() } })
        }
    }
}
