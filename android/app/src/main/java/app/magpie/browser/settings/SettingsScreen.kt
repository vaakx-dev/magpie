package app.magpie.browser.settings

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.magpie.browser.browser.Browser
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.SearchBox

private val CARD_SHAPE = RoundedCornerShape(12.dp)
private val ICON_SHAPE = RoundedCornerShape(8.dp)

private fun on(enabled: Boolean): String = if (enabled) "on" else "off"

private fun summary(category: Category, browser: Browser): String {
    val settings = browser.settings
    return when (category) {
        Category.General -> "${browser.spaces.saved.size} spaces · Swipe tabs ${on(settings.swipeTabs)}"
        Category.Appearance -> "${settings.theme.label} theme · ${settings.menuSide.placement}"
        Category.Search -> settings.searchEngine.label
        Category.Privacy -> "Ad blocker ${on(settings.privacy.adBlock)} · Tracking protection ${on(settings.privacy.trackingProtection)}"
        Category.Passwords -> settings.passwordService.label
        Category.Addons -> "Distraction Blocker · Background play ${on(settings.backgroundPlay)}"
        Category.About -> "Version ${browser.updates.version} · ${settings.updateChannel.label} updates"
    }
}

@Composable
private fun CategoryCard(icon: ImageVector, title: String, detail: String, onOpen: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier.fillMaxWidth().clip(CARD_SHAPE).background(colors.surface).clickable(onClick = onOpen)
            .padding(start = 12.dp, end = 10.dp, top = 13.dp, bottom = 13.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Box(Modifier.size(38.dp).clip(ICON_SHAPE).background(colors.surfaceVariant), contentAlignment = Alignment.Center) {
            Icon(icon, contentDescription = null, Modifier.size(20.dp), tint = colors.onSurface)
        }
        Column(Modifier.weight(1f)) {
            Text(title, fontSize = 14.5.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(detail, fontSize = 12.sp, color = colors.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        Icon(Icons.AutoMirrored.Outlined.KeyboardArrowRight, contentDescription = null, tint = colors.onSurfaceVariant)
    }
}

@Composable
private fun Index(browser: Browser, onOpen: (Category) -> Unit) {
    var query by rememberSaveable { mutableStateOf("") }
    SearchBox(query, "Search settings", { query = it }, Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp))
    Column(
        Modifier.verticalScroll(rememberScrollState()).padding(start = 12.dp, end = 12.dp, bottom = 16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        val typed = query.trim()
        if (typed.isEmpty()) {
            Category.entries.forEach { category -> CategoryCard(category.icon, category.label, summary(category, browser)) { onOpen(category) } }
            return@Column
        }
        val matches = search(typed)
        if (matches.isEmpty()) {
            Note("No settings match “$typed”")
            return@Column
        }
        matches.forEach { match -> CategoryCard(match.category.icon, match.term, match.category.label) { onOpen(match.category) } }
    }
}

@Composable
fun SettingsScreen(browser: Browser, onBack: () -> Unit, onPasswords: () -> Unit) {
    var category by rememberSaveable { mutableStateOf<Category?>(null) }
    val close = { category = null }
    BackHandler { category?.let { close() } ?: onBack() }
    Surface(color = MaterialTheme.colorScheme.background, contentColor = MaterialTheme.colorScheme.onSurface) {
        Column(Modifier.fillMaxSize().systemBarsPadding().imePadding()) {
            val open = category
            if (open == null) {
                Bar("Settings", onBack)
                Index(browser) { category = it }
                return@Column
            }
            Bar(open.label, close)
            Column(Modifier.verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
                Page(open, browser, onPasswords, onBack)
            }
        }
    }
}
