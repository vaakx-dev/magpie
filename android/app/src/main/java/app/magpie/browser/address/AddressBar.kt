package app.magpie.browser.address

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material.icons.outlined.Menu
import androidx.compose.material.icons.outlined.NoEncryption
import androidx.compose.material.icons.outlined.Refresh
import androidx.compose.material.icons.outlined.Search
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import app.magpie.browser.browser.Browser
import app.magpie.browser.browser.Hero
import app.magpie.browser.browser.PROGRESS_DONE
import app.magpie.browser.browser.Tab
import app.magpie.browser.settings.MenuSide
import app.magpie.browser.space.Profile
import app.magpie.browser.theme.Count
import app.magpie.browser.theme.Outline
import app.magpie.browser.theme.host

const val PLACEHOLDER = "Search or type URL"

private const val TEXT_FOLLOW = 0.4f
private const val TEXT_FOLLOW_AT_END = 0.15f
private const val NEXT_ALPHA = 0.55f
private const val OTHER_ALPHA = 0.35f
private val SHAPE = RoundedCornerShape(8.dp)
private val BAR_HEIGHT = 58.dp
private val FIELD_HEIGHT = 40.dp
private val HINT_MAX_WIDTH = 250.dp

private fun page(tab: Tab?): Tab? = tab?.takeUnless { it.blank }

private fun security(tab: Tab?): Pair<ImageVector, String> {
    val current = page(tab) ?: return Icons.Outlined.Search to "Search"
    if (current.secure) return Icons.Outlined.Lock to "Secure connection"
    if (current.url.startsWith("http:")) return Icons.Outlined.NoEncryption to "Not secure"
    return Icons.Outlined.Search to "Search"
}

private fun step(across: Float, thresholdPx: Float): Int? = when {
    across > thresholdPx -> -1
    across < -thresholdPx -> 1
    else -> null
}

private fun direction(across: Float): Int = if (across > 0) -1 else 1

private fun Browser.across(swipe: Swipe): Boolean = swipe.axis == Axis.Across && settings.swipeTabs

private fun Browser.swipeTo(across: Float, thresholdPx: Float) {
    step(across, thresholdPx)?.let(::neighbour)?.let(::select)
}

@Composable
private fun follow(target: Float, swipe: Swipe): Float {
    val spec = if (swipe.axis == null) spring() else snap<Float>()
    val value by animateFloatAsState(target, spec, label = "swipe")
    return value
}

@Composable
private fun ReloadButton(tab: Tab) {
    val loading = tab.progress < PROGRESS_DONE
    IconButton(
        onClick = { if (loading) tab.session.stop() else tab.session.reload() },
        modifier = Modifier.size(32.dp),
        shape = SHAPE,
    ) {
        Icon(
            if (loading) Icons.Outlined.Close else Icons.Outlined.Refresh,
            contentDescription = if (loading) "Stop" else "Reload",
            Modifier.size(18.dp),
            tint = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun MenuButton(onClick: () -> Unit) {
    IconButton(onClick = onClick, shape = SHAPE) {
        Icon(Icons.Outlined.Menu, contentDescription = "Menu", Modifier.size(22.dp), tint = MaterialTheme.colorScheme.onSurface)
    }
}

@Composable
private fun UrlField(browser: Browser, swipe: Swipe, hero: Hero, onType: () -> Unit, modifier: Modifier) {
    val colors = MaterialTheme.colorScheme
    val tab = page(browser.tab)
    val private = browser.space.profile == Profile.Private
    val (icon, description) = if (private) Icons.Outlined.VisibilityOff to "Private" else security(browser.tab)
    val tabThresholdPx = with(LocalDensity.current) { SWIPE_TAB_DISTANCE.toPx() }
    val gesture = rememberUpdatedState(
        Gesture(
            onTap = onType,
            onMove = { total ->
                swipe.move(total)
                if (swipe.axis == Axis.Up) hero.pull(total)
            },
            onRelease = { total, velocity ->
                if (browser.across(swipe)) browser.swipeTo(total.x, tabThresholdPx)
                if (swipe.axis == Axis.Up) hero.release(total, velocity)
                swipe.reset()
            },
        ),
    )
    val follows = if (browser.neighbour(direction(swipe.offset.x)) != null) TEXT_FOLLOW else TEXT_FOLLOW_AT_END
    val shift = follow(if (browser.across(swipe)) swipe.offset.x * follows else 0f, swipe)
    Row(
        modifier.height(FIELD_HEIGHT).clip(SHAPE).background(colors.surfaceVariant).padding(end = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            Modifier.weight(1f).fillMaxHeight().gesture(gesture).padding(start = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(icon, description, Modifier.size(16.dp), tint = colors.onSurfaceVariant)
            Text(
                tab?.let { host(it.url) } ?: PLACEHOLDER,
                color = if (tab == null) colors.onSurfaceVariant else colors.onSurface,
                fontSize = 14.5.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f).graphicsLayer { translationX = shift },
            )
        }
        tab?.let { ReloadButton(it) }
    }
}

@Composable
fun AddressBar(browser: Browser, swipe: Swipe, hero: Hero, onType: () -> Unit, onMenu: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val side = browser.settings.menuSide
    val private = browser.space.profile == Profile.Private
    Row(
        Modifier.fillMaxWidth()
            .background(colors.surface)
            .navigationBarsPadding()
            .height(BAR_HEIGHT)
            .padding(horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        if (side == MenuSide.Left) MenuButton(onMenu)
        UrlField(browser, swipe, hero, onType, Modifier.weight(1f))
        IconButton(onClick = hero::open, shape = SHAPE) {
            Count(browser.spaceTabs.size, if (private) Outline.Dashed else Outline.Solid)
        }
        if (side == MenuSide.Right) MenuButton(onMenu)
    }
}

private fun hint(browser: Browser, swipe: Swipe): Pair<String, Tab?> {
    val target = browser.neighbour(direction(swipe.offset.x)) ?: return "No more tabs" to null
    return (if (swipe.offset.x > 0) "‹  ${target.label}" else "${target.label}  ›") to target
}

@Composable
private fun Dots(browser: Browser, target: Tab?) {
    val colors = MaterialTheme.colorScheme
    Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        browser.spaceTabs.forEach { tab ->
            val color = when (tab) {
                browser.tab -> colors.onSurface
                target -> colors.onSurface.copy(alpha = NEXT_ALPHA)
                else -> colors.onSurfaceVariant.copy(alpha = OTHER_ALPHA)
            }
            Box(Modifier.size(12.dp, 3.dp).clip(RoundedCornerShape(2.dp)).background(color))
        }
    }
}

@Composable
fun SwipeHint(browser: Browser, swipe: Swipe, modifier: Modifier) {
    if (!browser.across(swipe)) return
    val (label, target) = hint(browser, swipe)
    Surface(modifier.padding(bottom = 10.dp).widthIn(max = HINT_MAX_WIDTH), shape = SHAPE, shadowElevation = 6.dp) {
        Column(Modifier.padding(horizontal = 12.dp, vertical = 7.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(label, fontSize = 12.5.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Box(Modifier.padding(top = 6.dp)) { Dots(browser, target) }
        }
    }
}
