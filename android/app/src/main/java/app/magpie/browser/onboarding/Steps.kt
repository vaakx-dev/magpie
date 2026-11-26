package app.magpie.browser.onboarding

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Language
import androidx.compose.material.icons.outlined.Password
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.LifecycleResumeEffect
import app.magpie.browser.R
import app.magpie.browser.focus.Distraction
import app.magpie.browser.focus.Focus
import app.magpie.browser.password.changeAutofillService
import app.magpie.browser.privacy.Privacy
import app.magpie.browser.settings.Card
import app.magpie.browser.settings.Item
import app.magpie.browser.settings.Link
import app.magpie.browser.settings.MenuSide
import app.magpie.browser.settings.PickItem
import app.magpie.browser.settings.SearchEngine
import app.magpie.browser.settings.Settings
import app.magpie.browser.settings.SwitchItem
import app.magpie.browser.settings.ThemeMode
import app.magpie.browser.sponsorblock.ENABLED
import app.magpie.browser.theme.Choices
import app.magpie.browser.theme.Section

private val THEMES = listOf(ThemeMode.System, ThemeMode.Dark, ThemeMode.Light)
private val DEFAULT_DISTRACTIONS = Distraction.entries.filter { it.blockedByDefault }.toSet()
private val ICON_SIZE = 96.dp

private fun Focus.blocksDefaults(): Boolean = blocked.containsAll(DEFAULT_DISTRACTIONS)

private fun Privacy.blocking(): Boolean = adBlock && stripTracking && trackingProtection

@Composable
private fun Heading(text: String) {
    Text(
        text,
        modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
        style = MaterialTheme.typography.headlineSmall,
        fontWeight = FontWeight.SemiBold,
    )
}

@Composable
fun Welcome() {
    Column(
        Modifier.fillMaxWidth().padding(horizontal = 20.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Image(
            painterResource(R.drawable.ic_launcher_foreground),
            contentDescription = null,
            colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onSurface),
            modifier = Modifier.size(ICON_SIZE),
        )
        Text("Welcome to Magpie", style = MaterialTheme.typography.headlineSmall, fontWeight = FontWeight.SemiBold)
        Text(
            "Fast, private, distraction-free browsing.",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
fun Look(settings: Settings) {
    Heading("Look & layout")
    Section("Theme")
    Choices(THEMES, settings.theme, ThemeMode::label) { settings.theme = it }
    Section("Menu side")
    Choices(MenuSide.entries, settings.menuSide, MenuSide::label) { settings.menuSide = it }
}

@Composable
private fun Protection(settings: Settings) {
    val sponsorBlock = settings.sponsorBlock
    Card(
        listOfNotNull(
            {
                SwitchItem(null, "Ad & tracker blocker", checked = settings.privacy.blocking(), onChange = { on ->
                    settings.privacy = Privacy(adBlock = on, stripTracking = on, trackingProtection = on)
                })
            },
            {
                val focus = settings.focus
                SwitchItem(null, "YouTube Shorts & distraction blocker", checked = focus.blocksDefaults(), onChange = { on ->
                    settings.focus = Focus(if (on) focus.blocked + DEFAULT_DISTRACTIONS else focus.blocked - DEFAULT_DISTRACTIONS)
                })
            },
            sponsorBlock.view?.let { view ->
                {
                    SwitchItem(null, "SponsorBlock", checked = view.enabled, onChange = { on ->
                        settings.sponsorBlock = sponsorBlock.switch(ENABLED, on)
                    })
                }
            },
            { SwitchItem(null, "Background play", checked = settings.backgroundPlay, onChange = { settings.backgroundPlay = it }) },
        ),
    )
}

@Composable
fun SearchAndFocus(settings: Settings) {
    Heading("Search & focus")
    Section("Search engine")
    Card(
        SearchEngine.entries.filter { it.template != null }.map { engine ->
            { PickItem(engine.label, settings.search.engine == engine) { settings.search = settings.search.copy(engine = engine) } }
        },
    )
    Section("Built-in protection")
    Protection(settings)
}

@Composable
fun Stuff() {
    val context = LocalContext.current
    var autofill by remember { mutableStateOf(autofillOn(context)) }
    val defaultBrowser = rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) {}
    LifecycleResumeEffect(Unit) {
        autofill = autofillOn(context)
        onPauseOrDispose {}
    }
    Heading("Bring your stuff")
    Section("All optional")
    Card(
        listOf(
            { Item(Icons.Outlined.Password, "System autofill", onClick = {}, detail = if (autofill) "Set up" else "Not set up") },
            { Link(null, "Open autofill settings", onClick = { changeAutofillService(context) }) },
        ),
    )
    Card(listOf { Link(Icons.Outlined.Language, "Set as default browser", onClick = { defaultBrowser.launch(defaultBrowserRequest(context)) }) })
}
