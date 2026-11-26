package app.magpie.browser.onboarding

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import app.magpie.browser.browser.Browser

private enum class Step(val action: String) {
    Welcome("Get started"),
    Look("Next"),
    Search("Next"),
    Stuff("Start browsing");

    val next: Step? get() = entries.getOrNull(ordinal + 1)
}

private val DOT_SIZE = 6.dp
private val CURRENT_DOT_WIDTH = 18.dp
private val DOT_SHAPE = RoundedCornerShape(3.dp)
private val BUTTON_SHAPE = RoundedCornerShape(8.dp)
private val BUTTON_HEIGHT = 48.dp

private fun Browser.startBrowsing() {
    settings.onboardingDone = true
    if (tab == null) open()
}

@Composable
private fun Dots(current: Step) {
    Row(horizontalArrangement = Arrangement.spacedBy(DOT_SIZE)) {
        Step.entries.forEach { step ->
            val shown = step == current
            Box(
                Modifier.size(if (shown) CURRENT_DOT_WIDTH else DOT_SIZE, DOT_SIZE)
                    .clip(DOT_SHAPE)
                    .background(if (shown) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.surfaceVariant),
            )
        }
    }
}

@Composable
private fun Top(step: Step, onSkip: () -> Unit) {
    Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.weight(1f)) { Dots(step) }
        TextButton(onClick = onSkip) { Text("Skip", color = MaterialTheme.colorScheme.onSurfaceVariant) }
    }
}

@Composable
private fun Scrolling(modifier: Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier.verticalScroll(rememberScrollState()), content = content)
}

@Composable
private fun ColumnScope.Body(step: Step, browser: Browser) {
    val body = Modifier.weight(1f).fillMaxWidth()
    when (step) {
        Step.Welcome -> Box(body, contentAlignment = Alignment.Center) { Welcome() }
        Step.Look -> Scrolling(body) { Look(browser.settings) }
        Step.Search -> Scrolling(body) { SearchAndFocus(browser.settings) }
        Step.Stuff -> Scrolling(body) { Stuff() }
    }
}

@Composable
private fun Primary(text: String, onClick: () -> Unit) {
    Button(
        onClick = onClick,
        shape = BUTTON_SHAPE,
        modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 12.dp).height(BUTTON_HEIGHT),
    ) {
        Text(text)
    }
}

@Composable
fun Onboarding(browser: Browser) {
    var step by rememberSaveable { mutableStateOf(Step.Welcome) }
    val finish = { browser.startBrowsing() }
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).systemBarsPadding()) {
        Top(step, onSkip = finish)
        Body(step, browser)
        Primary(step.action) { step.next?.let { step = it } ?: finish() }
    }
}
