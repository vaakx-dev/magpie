package app.magpie.browser.prompt

import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import app.magpie.browser.theme.Field
import app.magpie.browser.theme.SecretField
import org.mozilla.geckoview.AllowOrDeny
import org.mozilla.geckoview.GeckoSession.PromptDelegate.AlertPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.AuthPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.BeforeUnloadPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.ButtonPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.ChoicePrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.FilePrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.TextPrompt

private fun AuthPrompt.passwordOnly(): Boolean = authOptions.flags and AuthPrompt.AuthOptions.Flags.ONLY_PASSWORD != 0

@Composable
private fun Alert(prompt: Prompt, request: AlertPrompt, site: String) {
    Dialog(
        title = request.title ?: "$site says",
        onDismiss = prompt::dismiss,
        confirm = { Confirm("OK", onClick = prompt::dismiss) },
    ) { Message(request.message.orEmpty()) }
}

@Composable
private fun Question(prompt: Prompt, request: ButtonPrompt, site: String) {
    Dialog(
        title = request.title ?: "$site says",
        onDismiss = prompt::dismiss,
        confirm = { Confirm("OK") { prompt.answer(request.confirm(ButtonPrompt.Type.POSITIVE)) } },
        cancel = { Cancel("Cancel") { prompt.answer(request.confirm(ButtonPrompt.Type.NEGATIVE)) } },
    ) { Message(request.message.orEmpty()) }
}

@Composable
private fun Input(prompt: Prompt, request: TextPrompt, site: String) {
    val text = rememberTextFieldState(request.defaultValue.orEmpty())
    val submit = { prompt.answer(request.confirm(text.text.toString())) }
    Dialog(
        title = request.title ?: "$site says",
        onDismiss = prompt::dismiss,
        confirm = { Confirm("OK", onClick = submit) },
        cancel = { Cancel("Cancel", onClick = prompt::dismiss) },
    ) {
        request.message?.let { Message(it) }
        Field(text, "", onDone = submit)
    }
}

@Composable
private fun SignIn(prompt: Prompt, request: AuthPrompt) {
    val username = rememberTextFieldState(request.authOptions.username.orEmpty())
    val password = rememberTextFieldState(request.authOptions.password.orEmpty())
    val passwordOnly = request.passwordOnly()
    val submit = {
        val response = if (passwordOnly) request.confirm(password.text.toString())
        else request.confirm(username.text.toString(), password.text.toString())
        prompt.answer(response)
    }
    Dialog(
        title = request.title ?: "Sign in",
        onDismiss = prompt::dismiss,
        confirm = { Confirm("Sign in", onClick = submit) },
        cancel = { Cancel("Cancel", onClick = prompt::dismiss) },
    ) {
        request.message?.let { Message(it) }
        if (!passwordOnly) Field(username, "Username")
        SecretField(password, "Password")
    }
}

@Composable
private fun Leave(prompt: Prompt, request: BeforeUnloadPrompt) {
    Dialog(
        title = "Leave this page?",
        onDismiss = prompt::dismiss,
        confirm = { Confirm("Leave") { prompt.answer(request.confirm(AllowOrDeny.ALLOW)) } },
        cancel = { Cancel("Stay") { prompt.answer(request.confirm(AllowOrDeny.DENY)) } },
    ) { Message("Changes you made may not be saved.") }
}

@Composable
fun PromptView(prompt: Prompt, site: String) = key(prompt) {
    when (val request = prompt.request) {
        is AlertPrompt -> Alert(prompt, request, site)
        is ButtonPrompt -> Question(prompt, request, site)
        is TextPrompt -> Input(prompt, request, site)
        is AuthPrompt -> SignIn(prompt, request)
        is BeforeUnloadPrompt -> Leave(prompt, request)
        is ChoicePrompt -> ChoiceSheet(prompt, request)
        is FilePrompt -> FilePicker(prompt, request)
    }
}
