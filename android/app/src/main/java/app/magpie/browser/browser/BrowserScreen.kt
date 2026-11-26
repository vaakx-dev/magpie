package app.magpie.browser.browser

import android.view.View
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.imeAnimationTarget
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import app.magpie.browser.R
import app.magpie.browser.address.AddressBar
import app.magpie.browser.address.Swipe
import app.magpie.browser.address.SwipeHint
import app.magpie.browser.address.Typing
import app.magpie.browser.download.DownloadsSheet
import app.magpie.browser.favorite.BookmarksSheet
import app.magpie.browser.find.FindBar
import app.magpie.browser.history.HistorySheet
import app.magpie.browser.menu.MenuPopup
import app.magpie.browser.password.PasswordSheet
import app.magpie.browser.settings.SettingsScreen
import app.magpie.browser.update.UpdateBanner
import org.mozilla.geckoview.GeckoView

enum class Panel { Typing, Menu, Find, History, Bookmarks, Downloads, Passwords, Settings }

private const val WELCOME_ALPHA = 0.35f

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun Modifier.aboveKeyboard(chrome: Dp) =
    consumeWindowInsets(PaddingValues(bottom = chrome)).windowInsetsPadding(WindowInsets.imeAnimationTarget)

@Composable
fun BrowserScreen(browser: Browser) {
    var panel by remember { mutableStateOf<Panel?>(null) }
    val close = { panel = null }
    val tab = browser.tab
    val scope = rememberCoroutineScope()
    val density = LocalDensity.current
    val hero = remember(browser) { Hero(browser, scope, density) }
    LaunchedEffect(tab?.id) { if (tab?.blank == true) panel = Panel.Typing }
    if (panel == Panel.Settings) return SettingsScreen(browser, onBack = close, onPasswords = { panel = Panel.Passwords })
    val swipe = remember { Swipe() }
    val fullScreen = tab?.fullScreen == true
    var chrome by remember { mutableStateOf(0.dp) }
    val insets = if (fullScreen) Modifier else Modifier.statusBarsPadding()
    val page = if (fullScreen) Modifier else Modifier.aboveKeyboard(chrome)
    BackHandler(enabled = tab?.canGoBack == true) { tab?.session?.goBack() }
    BackHandler(enabled = fullScreen) { tab?.session?.exitFullScreen() }
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).then(insets)) {
        Box(Modifier.weight(1f).fillMaxWidth().then(page).onGloballyPositioned { hero.page = it.bounds() }) {
            tab?.let { Page(it, onView = { created -> hero.view = created }) }
            if (!fullScreen) {
                if (tab?.blank != false) Welcome()
                Progress(tab?.progress ?: PROGRESS_DONE)
                SwipeHint(browser, swipe, Modifier.align(Alignment.BottomCenter))
            }
        }
        if (!fullScreen) Column(Modifier.onSizeChanged { chrome = with(density) { it.height.toDp() } }) {
            UpdateBanner(browser.updates)
            when {
                panel == Panel.Find && tab != null ->
                    Box(Modifier.background(MaterialTheme.colorScheme.surface).windowInsetsPadding(WindowInsets.ime.union(WindowInsets.navigationBars))) {
                        FindBar(tab, onClose = close)
                    }
                else -> Box(Modifier.graphicsLayer { alpha = hero.barAlpha; translationY = hero.barShift }) {
                    AddressBar(browser, swipe, hero, onType = { panel = Panel.Typing }, onMenu = { panel = Panel.Menu })
                    if (panel == Panel.Menu) MenuPopup(browser, onOpen = { panel = it }, onDismiss = close)
                }
            }
        }
    }
    if (hero.shown) {
        TabSwitcher(browser, hero) {
            hero.hide()
            panel = Panel.Settings
        }
    }
    when (panel) {
        Panel.Typing -> Typing(browser, onDismiss = close)
        Panel.History -> HistorySheet(browser, onDismiss = close)
        Panel.Bookmarks -> BookmarksSheet(browser, onDismiss = close)
        Panel.Downloads -> DownloadsSheet(browser.downloads, onDismiss = close)
        Panel.Passwords -> PasswordSheet(browser, onDismiss = close)
        Panel.Menu, Panel.Find, Panel.Settings, null -> Unit
    }
}

private fun autofillable(view: GeckoView): GeckoView = view.apply { importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_YES }

@Composable
private fun Page(tab: Tab, onView: (GeckoView?) -> Unit) {
    AndroidView(
        factory = { context -> autofillable(GeckoView(context)).also(onView) },
        update = { view ->
            if (view.session != tab.session) {
                view.releaseSession()
                view.setSession(tab.session)
            }
        },
        onRelease = {
            it.releaseSession()
            onView(null)
        },
        modifier = Modifier.fillMaxSize(),
    )
}

@Composable
private fun Welcome() {
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background), contentAlignment = Alignment.Center) {
        Image(
            painterResource(R.drawable.ic_launcher_foreground),
            contentDescription = null,
            colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onSurface.copy(alpha = WELCOME_ALPHA)),
            modifier = Modifier.size(160.dp),
        )
    }
}

@Composable
private fun Progress(progress: Int) {
    if (progress >= PROGRESS_DONE) return
    LinearProgressIndicator(
        progress = { progress / PROGRESS_DONE.toFloat() },
        color = MaterialTheme.colorScheme.onSurface,
        trackColor = Color.Transparent,
        drawStopIndicator = {},
        gapSize = 0.dp,
        modifier = Modifier.fillMaxWidth().height(2.dp),
    )
}
