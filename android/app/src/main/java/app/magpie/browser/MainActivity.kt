package app.magpie.browser

import android.content.Intent
import android.content.pm.ActivityInfo
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import android.view.WindowManager.LayoutParams.FLAG_SECURE
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import app.magpie.browser.address.link
import app.magpie.browser.browser.BrowserScreen
import app.magpie.browser.external.confirmAppLink
import app.magpie.browser.favicon.LocalFavicons
import app.magpie.browser.space.Profile
import app.magpie.browser.theme.MagpieTheme
import app.magpie.browser.theme.dark

class MainActivity : ComponentActivity() {
    private val browser get() = (application as Magpie).browser

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (savedInstanceState == null) openLink(intent)
        setContent {
            MagpieTheme(browser.settings.theme) {
                SystemBars()
                FullScreen()
                PrivateScreen()
                AppLinkPrompt()
                CompositionLocalProvider(LocalFavicons provides browser.favicons) { BrowserScreen(browser) }
            }
        }
    }

    override fun onStop() {
        super.onStop()
        browser.background()
        browser.flush()
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        openLink(intent)
    }

    @Composable
    private fun SystemBars() {
        val dark = browser.settings.theme.dark()
        LaunchedEffect(dark) {
            val style = if (dark) SystemBarStyle.dark(Color.TRANSPARENT) else SystemBarStyle.light(Color.TRANSPARENT, Color.TRANSPARENT)
            enableEdgeToEdge(statusBarStyle = style, navigationBarStyle = style)
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                window.isStatusBarContrastEnforced = false
                window.isNavigationBarContrastEnforced = false
            }
        }
    }

    @Composable
    private fun FullScreen() {
        val tab = browser.tab
        val fullScreen = tab?.fullScreen == true
        val orientation = tab?.orientation ?: ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
        LaunchedEffect(fullScreen) {
            val bars = WindowCompat.getInsetsController(window, window.decorView)
            bars.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            if (fullScreen) bars.hide(WindowInsetsCompat.Type.systemBars()) else bars.show(WindowInsetsCompat.Type.systemBars())
        }
        LaunchedEffect(orientation) { requestedOrientation = orientation }
    }

    @Composable
    private fun PrivateScreen() {
        val private = browser.space.profile == Profile.Private
        SideEffect {
            if (private) window.addFlags(FLAG_SECURE) else window.clearFlags(FLAG_SECURE)
        }
    }

    @Composable
    private fun AppLinkPrompt() {
        val tab = browser.tab ?: return
        val uri = tab.appLink ?: return
        DisposableEffect(tab, uri) {
            val dialog = confirmAppLink(this@MainActivity, open = tab::openAppLink, dismiss = tab::dismissAppLink)
            onDispose { dialog.dismiss() }
        }
    }

    private fun openLink(intent: Intent) {
        val link = link(intent.action, intent.dataString) ?: return
        browser.open().load(browser.clean(link))
    }
}
