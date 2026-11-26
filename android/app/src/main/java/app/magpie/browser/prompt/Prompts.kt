package app.magpie.browser.prompt

import androidx.compose.runtime.mutableStateListOf
import org.mozilla.geckoview.GeckoResult
import org.mozilla.geckoview.GeckoSession
import org.mozilla.geckoview.GeckoSession.PromptDelegate.AlertPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.AuthPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.BasePrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.BeforeUnloadPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.ButtonPrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.ChoicePrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.FilePrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.PromptInstanceDelegate
import org.mozilla.geckoview.GeckoSession.PromptDelegate.PromptResponse
import org.mozilla.geckoview.GeckoSession.PromptDelegate.TextPrompt

class Prompt(val request: BasePrompt, private val result: GeckoResult<PromptResponse>, private val done: (Prompt) -> Unit) {
    fun answer(response: PromptResponse) {
        done(this)
        result.complete(response)
    }

    fun dismiss() = answer(request.dismiss())
}

class Prompts : GeckoSession.PromptDelegate {
    private val waiting = mutableStateListOf<Prompt>()

    val current: Prompt? get() = waiting.firstOrNull()

    private fun ask(request: BasePrompt): GeckoResult<PromptResponse> {
        val result = GeckoResult<PromptResponse>()
        val prompt = Prompt(request, result, waiting::remove)
        request.delegate = object : PromptInstanceDelegate {
            override fun onPromptDismiss(prompt: BasePrompt) {
                waiting.removeAll { it.request == prompt }
            }
        }
        waiting += prompt
        return result
    }

    override fun onAlertPrompt(session: GeckoSession, prompt: AlertPrompt) = ask(prompt)

    override fun onButtonPrompt(session: GeckoSession, prompt: ButtonPrompt) = ask(prompt)

    override fun onTextPrompt(session: GeckoSession, prompt: TextPrompt) = ask(prompt)

    override fun onAuthPrompt(session: GeckoSession, prompt: AuthPrompt) = ask(prompt)

    override fun onBeforeUnloadPrompt(session: GeckoSession, prompt: BeforeUnloadPrompt) = ask(prompt)

    override fun onChoicePrompt(session: GeckoSession, prompt: ChoicePrompt) = ask(prompt)

    override fun onFilePrompt(session: GeckoSession, prompt: FilePrompt) = ask(prompt)
}
