package app.magpie.browser.prompt

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import app.magpie.browser.menu.Divider
import app.magpie.browser.menu.Glyph
import app.magpie.browser.menu.Heading
import app.magpie.browser.menu.ICON_SIZE
import app.magpie.browser.menu.Line
import app.magpie.browser.theme.GUTTER
import app.magpie.browser.theme.Sheet
import app.magpie.browser.theme.SolidButton
import app.magpie.browser.theme.Title
import org.mozilla.geckoview.GeckoSession.PromptDelegate.ChoicePrompt
import org.mozilla.geckoview.GeckoSession.PromptDelegate.ChoicePrompt.Choice

private val LIST_PADDING = 10.dp

private fun flat(choices: Array<Choice>): List<Choice> =
    choices.flatMap { choice -> listOf(choice) + choice.items?.let(::flat).orEmpty() }

@Composable
private fun Option(choice: Choice, picked: Boolean, onPick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Line(
        leading = { Box(Modifier.size(ICON_SIZE)) { if (picked) Glyph(Icons.Outlined.Check, colors.onSurface) } },
        label = choice.label,
        modifier = Modifier.clickable(enabled = !choice.disabled, onClick = onPick),
        color = if (choice.disabled) colors.outline else colors.onSurface,
    )
}

@Composable
private fun Entry(choice: Choice, picked: Boolean, onPick: () -> Unit) {
    when {
        choice.separator -> Divider()
        choice.items != null -> Heading(choice.label)
        else -> Option(choice, picked, onPick)
    }
}

@Composable
fun ChoiceSheet(prompt: Prompt, request: ChoicePrompt) {
    val entries = remember(request) { flat(request.choices) }
    val multiple = request.type == ChoicePrompt.Type.MULTIPLE
    var picked by remember(request) { mutableStateOf(entries.filter { it.selected }.map { it.id }.toSet()) }
    val toggle = { choice: Choice -> picked = if (choice.id in picked) picked - choice.id else picked + choice.id }
    val pick = { choice: Choice -> if (multiple) toggle(choice) else prompt.answer(request.confirm(choice)) }
    val done = { prompt.answer(request.confirm(entries.filter { it.id in picked }.toTypedArray())) }
    Sheet(onDismiss = prompt::dismiss) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Title(request.title ?: request.message ?: if (multiple) "Choose options" else "Choose an option")
            if (multiple) SolidButton("Done", onClick = done, modifier = Modifier.padding(end = GUTTER))
        }
        LazyColumn(Modifier.weight(1f, fill = false).padding(horizontal = LIST_PADDING)) {
            items(entries) { choice -> Entry(choice, choice.id in picked) { pick(choice) } }
        }
    }
}
