package app.magpie.browser.browser

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.Animatable
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Add
import androidx.compose.material.icons.outlined.AddToPhotos
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material.icons.outlined.CheckCircleOutline
import androidx.compose.material.icons.outlined.Close
import androidx.compose.material.icons.outlined.DeleteSweep
import androidx.compose.material.icons.outlined.GridView
import androidx.compose.material.icons.outlined.MoreVert
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.GraphicsLayerScope
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.layout
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.unit.dp
import androidx.compose.ui.util.lerp
import androidx.compose.ui.unit.sp
import app.magpie.browser.favicon.Favicon
import app.magpie.browser.favicon.retention
import app.magpie.browser.menu.Divider
import app.magpie.browser.menu.Floating
import app.magpie.browser.menu.Item
import app.magpie.browser.settings.ConfirmClearBrowsingData
import app.magpie.browser.settings.MenuSide
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import app.magpie.browser.space.SpaceDialog
import app.magpie.browser.theme.CARD_SHAPE
import app.magpie.browser.theme.Confirmation
import app.magpie.browser.theme.Count
import app.magpie.browser.theme.Note
import app.magpie.browser.theme.SearchBox
import kotlinx.coroutines.launch
import kotlin.math.abs
import kotlin.math.roundToInt
import kotlin.math.sign

private enum class Mode(val search: String) { Tabs("Search your tabs"), Spaces("Search your spaces"), Private("Search your tabs") }

private const val MAX_SWIPE_FADE = 0.7f
private const val SPACE_MARKS = 5
private const val SETTLE_SCALE = 1.02f
private const val MUTED_ALPHA = 0.6f
private val PREVIEW_CORNER = 8.dp
private val SHAPE = RoundedCornerShape(PREVIEW_CORNER)
private val SEGMENT_SHAPE = RoundedCornerShape(6.dp)
private val CHECK_SHAPE = RoundedCornerShape(6.dp)
private val CARD_HEIGHT = 238.dp
private val SPACE_CARD_HEIGHT = 128.dp
private val CLOSE_DISTANCE = 90.dp
private val FADE_DISTANCE = 300.dp
private val THROW_DISTANCE = 400.dp

private fun Tab.matches(query: String): Boolean = title.contains(query, ignoreCase = true) || url.contains(query, ignoreCase = true)

private fun tabCount(count: Int): String = if (count == 1) "1 tab" else "$count tabs"

@Composable
private fun Check(selected: Boolean) {
    val colors = MaterialTheme.colorScheme
    Box(
        Modifier.padding(end = 8.dp).size(20.dp).clip(CHECK_SHAPE).background(if (selected) colors.onSurface else colors.surfaceVariant),
        contentAlignment = Alignment.Center,
    ) {
        if (selected) Icon(Icons.Outlined.Check, "Selected", Modifier.size(14.dp), tint = colors.surface)
    }
}

@Composable
private fun Preview(tab: Tab, modifier: Modifier) {
    Box(modifier.background(MaterialTheme.colorScheme.surfaceVariant), contentAlignment = Alignment.Center) {
        when (val preview = tab.preview) {
            null -> Favicon(tab.url, 44.dp, tab.retention)
            else -> Image(preview, null, Modifier.fillMaxSize(), contentScale = ContentScale.Crop, alignment = Alignment.TopCenter)
        }
    }
}

@Composable
private fun Modifier.landing(tab: Tab, hero: Hero, active: Boolean): Modifier = onGloballyPositioned { hero.place(tab, it.bounds()) }
    .graphicsLayer { alpha = if (active && hero.flying) 0f else 1f }
    .clip(SHAPE)

private fun GraphicsLayerScope.settle(hero: Hero, active: Boolean) {
    alpha *= hero.progress
    if (active) return
    scaleX = lerp(SETTLE_SCALE, 1f, hero.progress)
    scaleY = scaleX
}

@Composable
private fun TabCard(tab: Tab, hero: Hero, active: Boolean, selected: Boolean?, onTap: () -> Unit, onClose: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val density = LocalDensity.current
    val scope = rememberCoroutineScope()
    val offset = remember { Animatable(0f) }
    val closePx = with(density) { CLOSE_DISTANCE.toPx() }
    val fadePx = with(density) { FADE_DISTANCE.toPx() }
    val throwPx = with(density) { THROW_DISTANCE.toPx() }
    val drag = rememberDraggableState { delta -> scope.launch { offset.snapTo(offset.value + delta) } }
    val (header, ink) = if (active && selected == null) colors.onSurface to colors.surface else colors.surface to colors.onSurface
    DisposableEffect(tab) { onDispose { hero.forget(tab) } }
    Column(
        Modifier.height(CARD_HEIGHT)
            .graphicsLayer {
                translationX = offset.value
                alpha = 1f - (abs(offset.value) / fadePx).coerceAtMost(MAX_SWIPE_FADE)
                settle(hero, active)
            }
            .draggable(
                drag,
                Orientation.Horizontal,
                enabled = selected == null,
                onDragStopped = {
                    if (abs(offset.value) < closePx) {
                        offset.animateTo(0f)
                        return@draggable
                    }
                    offset.animateTo(sign(offset.value) * throwPx)
                    onClose()
                },
            )
            .clip(CARD_SHAPE)
            .background(colors.surface)
            .clickable(onClick = onTap),
    ) {
        Row(
            Modifier.height(36.dp).fillMaxWidth().background(header).padding(start = 10.dp, end = 2.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Favicon(tab.url, 20.dp, tab.retention)
            Text(
                tab.label,
                Modifier.weight(1f).padding(horizontal = 8.dp),
                color = ink,
                fontSize = 12.5.sp,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            when (selected) {
                null -> IconButton(onClick = onClose, modifier = Modifier.size(32.dp), shape = SHAPE) {
                    Icon(Icons.Outlined.Close, "Close tab", Modifier.size(17.dp), tint = ink)
                }
                else -> Check(selected)
            }
        }
        Preview(tab, Modifier.weight(1f).fillMaxWidth().padding(start = 4.dp, end = 4.dp, bottom = 4.dp).landing(tab, hero, active))
    }
}

private fun Modifier.flight(tab: Tab, hero: Hero, cornerPx: Float): Modifier = layout { measurable, constraints ->
    val bounds = hero.flight(tab)
    val placeable = measurable.measure(Constraints.fixed(bounds.width.roundToInt(), bounds.height.roundToInt()))
    layout(constraints.maxWidth, constraints.maxHeight) { placeable.place(bounds.left.roundToInt(), bounds.top.roundToInt()) }
}.graphicsLayer {
    alpha = if (hero.flying) 1f else 0f
    shape = RoundedCornerShape(cornerPx * hero.progress)
    clip = true
}

@Composable
private fun Ghost(tab: Tab, hero: Hero) {
    val cornerPx = with(LocalDensity.current) { PREVIEW_CORNER.toPx() }
    Preview(tab, Modifier.flight(tab, hero, cornerPx))
}

private fun DrawScope.backdrop(color: Color, hero: Hero) {
    val pageBottom = hero.page.bottom
    drawRect(color, size = Size(size.width, pageBottom))
    drawRect(color, Offset(0f, pageBottom), alpha = hero.dockAlpha)
}

@Composable
private fun SpaceCard(space: Space, tabs: List<Tab>, active: Boolean, onPick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val (fill, ink) = if (active) colors.onSurface to colors.surface else colors.surface to colors.onSurface
    Column(
        Modifier.height(SPACE_CARD_HEIGHT).clip(CARD_SHAPE).background(fill).clickable(onClick = onPick).padding(12.dp),
        verticalArrangement = Arrangement.SpaceBetween,
    ) {
        Column {
            Text(space.name, color = ink, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(tabCount(tabs.size), color = ink.copy(alpha = MUTED_ALPHA), fontSize = 12.sp)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) { tabs.take(SPACE_MARKS).forEach { Favicon(it.url, 22.dp, it.retention) } }
    }
}

@Composable
private fun NewSpaceCard(onCreate: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Column(
        Modifier.height(SPACE_CARD_HEIGHT).fillMaxWidth().clip(CARD_SHAPE).background(colors.surface).clickable(onClick = onCreate),
        verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Icon(Icons.Outlined.AddToPhotos, null, tint = colors.onSurfaceVariant)
        Text("New space", color = colors.onSurfaceVariant, fontSize = 13.sp)
    }
}

@Composable
private fun Segment(on: Boolean, description: String, onClick: () -> Unit, content: @Composable () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Box(
        Modifier.sizeIn(minWidth = 42.dp, minHeight = 34.dp)
            .clip(SEGMENT_SHAPE)
            .background(if (on) colors.onSurface else Color.Transparent)
            .clickable(onClickLabel = description, onClick = onClick)
            .padding(horizontal = 8.dp),
        contentAlignment = Alignment.Center,
    ) {
        CompositionLocalProvider(LocalContentColor provides if (on) colors.surface else colors.onSurfaceVariant, content = content)
    }
}

@Composable
private fun Segments(mode: Mode, tabCount: Int, onPick: (Mode) -> Unit) {
    Row(
        Modifier.clip(SHAPE).background(MaterialTheme.colorScheme.surface).padding(3.dp),
        horizontalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        Segment(mode == Mode.Tabs, "Tabs", { onPick(Mode.Tabs) }) { Count(tabCount) }
        Segment(mode == Mode.Spaces, "Spaces", { onPick(Mode.Spaces) }) { Icon(Icons.Outlined.GridView, "Spaces", Modifier.size(20.dp)) }
        Segment(mode == Mode.Private, "Private", { onPick(Mode.Private) }) { Icon(Icons.Outlined.VisibilityOff, "Private", Modifier.size(20.dp)) }
    }
}

@Composable
private fun Plus(onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Box(
        Modifier.size(52.dp, 40.dp).clip(SHAPE).background(colors.onSurface).clickable(onClickLabel = "New tab", onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Icon(Icons.Outlined.Add, "New tab", Modifier.size(22.dp), tint = colors.surface)
    }
}

@Composable
private fun SelectBar(count: Int, onDone: () -> Unit, onClose: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(Modifier.fillMaxWidth().height(52.dp).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
        IconButton(onClick = onDone, shape = SHAPE) { Icon(Icons.Outlined.Close, "Done", tint = colors.onSurface) }
        Text("$count selected", Modifier.weight(1f), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
        Box(
            Modifier.height(36.dp).clip(SHAPE).background(colors.onSurface.copy(alpha = if (count > 0) 1f else 0.35f))
                .clickable(enabled = count > 0, onClick = onClose).padding(horizontal = 14.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text("Close", color = colors.surface, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
        }
    }
}

private class Switcher(val browser: Browser) {
    var mode by mutableStateOf(if (browser.space.profile == Profile.Private) Mode.Private else Mode.Tabs)
    var query by mutableStateOf("")
    var selection by mutableStateOf<Set<Long>?>(null)
    var menu by mutableStateOf(false)
    var creating by mutableStateOf(false)
    var clearing by mutableStateOf(false)
    var closingAll by mutableStateOf(false)

    val space: Space get() = browser.space.takeUnless { it.profile == Profile.Private } ?: browser.spaces.saved.first()

    val tabs: List<Tab> get() = when (mode) {
        Mode.Private -> browser.tabs.filter { it.space.profile == Profile.Private }
        Mode.Tabs, Mode.Spaces -> browser.tabs.filter { it.space == space }
    }

    fun tabsIn(space: Space): List<Tab> = browser.tabs.filter { it.space == space }

    fun pick(mode: Mode) {
        this.mode = mode
        selection = null
    }

    fun select() {
        if (mode == Mode.Spaces) mode = Mode.Tabs
        selection = emptySet()
    }

    fun mark(tab: Tab) {
        val chosen = selection ?: return
        selection = if (tab.id in chosen) chosen - tab.id else chosen + tab.id
    }

    fun closeSelected() {
        val chosen = selection ?: return
        browser.tabs.filter { it.id in chosen }.forEach(browser::close)
        selection = null
    }

    fun open(): Tab = if (mode == Mode.Private) browser.openPrivate() else browser.open(space)
}

@Composable
private fun SwitcherMenu(switcher: Switcher, onSettings: () -> Unit, onDismiss: () -> Unit) {
    val browser = switcher.browser
    val choose = { action: () -> Unit ->
        switcher.menu = false
        action()
    }
    val openTab = { open: () -> Tab ->
        choose { open() }
        onDismiss()
    }
    Floating(browser.settings.menuSide, onDismiss = { switcher.menu = false }) {
        Item(Icons.Outlined.Add, "New tab", onClick = { openTab { browser.open(switcher.space) } })
        Item(Icons.Outlined.VisibilityOff, "New private tab", onClick = { openTab(browser::openPrivate) })
        Item(Icons.Outlined.AddToPhotos, "New space", onClick = { choose { switcher.creating = true } })
        Divider()
        Item(Icons.Outlined.Close, "Close all tabs", onClick = { choose { switcher.closingAll = true } }.takeIf { switcher.tabs.isNotEmpty() })
        Item(Icons.Outlined.CheckCircleOutline, "Select tabs", onClick = { choose(switcher::select) })
        Divider()
        Item(Icons.Outlined.DeleteSweep, "Delete browsing data", onClick = { choose { switcher.clearing = true } })
        Item(Icons.Outlined.Settings, "Settings", onClick = { choose(onSettings) })
    }
}

@Composable
private fun Toolbar(switcher: Switcher, onSettings: () -> Unit, onDismiss: () -> Unit) {
    val (menuEdge, segmentsEdge) = when (switcher.browser.settings.menuSide) {
        MenuSide.Left -> Alignment.CenterStart to Alignment.CenterEnd
        MenuSide.Right -> Alignment.CenterEnd to Alignment.CenterStart
    }
    Box(Modifier.fillMaxWidth().height(54.dp).padding(horizontal = 10.dp)) {
        Box(Modifier.align(segmentsEdge)) { Segments(switcher.mode, switcher.tabsIn(switcher.space).size, switcher::pick) }
        Box(Modifier.align(Alignment.Center)) {
            Plus {
                switcher.open()
                onDismiss()
            }
        }
        Box(Modifier.align(menuEdge)) {
            IconButton(onClick = { switcher.menu = true }, shape = SHAPE) {
                Icon(Icons.Outlined.MoreVert, "More", tint = MaterialTheme.colorScheme.onSurface)
            }
            if (switcher.menu) SwitcherMenu(switcher, onSettings, onDismiss)
        }
    }
}

@Composable
private fun Grid(switcher: Switcher, hero: Hero, modifier: Modifier) {
    val browser = switcher.browser
    val query = switcher.query.trim()
    val grid = rememberLazyGridState(initialFirstVisibleItemIndex = switcher.tabs.indexOf(browser.tab).coerceAtLeast(0))
    LazyVerticalGrid(
        GridCells.Fixed(2),
        modifier,
        grid,
        contentPadding = PaddingValues(10.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        if (switcher.mode == Mode.Spaces) {
            items(browser.spaces.saved.filter { it.name.contains(query, ignoreCase = true) }, key = Space::id) { space ->
                SpaceCard(space, switcher.tabsIn(space), space == switcher.space) {
                    browser.switch(space)
                    switcher.pick(Mode.Tabs)
                }
            }
            item { NewSpaceCard { switcher.creating = true } }
            return@LazyVerticalGrid
        }
        val shown = switcher.tabs.filter { it.matches(query) }
        if (shown.isEmpty()) item(span = { GridItemSpan(maxLineSpan) }) { Note(empty(switcher, query)) }
        items(shown, key = Tab::id) { tab ->
            val selected = switcher.selection?.let { tab.id in it }
            TabCard(
                tab,
                hero,
                active = tab == browser.tab,
                selected = selected,
                onTap = {
                    if (selected != null) {
                        switcher.mark(tab)
                        return@TabCard
                    }
                    hero.enter(tab)
                },
                onClose = { browser.close(tab) },
            )
        }
    }
}

private fun empty(switcher: Switcher, query: String): String = when {
    query.isNotEmpty() -> "No matching tabs"
    switcher.mode == Mode.Private -> "No private tabs. Private tabs aren't kept in history and close when you leave."
    else -> "No tabs in ${switcher.space.name}. Tap + to open one."
}

@Composable
private fun ConfirmCloseAll(switcher: Switcher) {
    val tabs = switcher.tabs
    val place = if (switcher.mode == Mode.Private) "Private" else switcher.space.name
    Confirmation(
        "Close all tabs?",
        "Closes ${tabCount(tabs.size)} in $place.",
        "Close all",
        onConfirm = {
            tabs.forEach(switcher.browser::close)
            switcher.closingAll = false
        },
        onDismiss = { switcher.closingAll = false },
    )
}

@Composable
fun TabSwitcher(browser: Browser, hero: Hero, notices: @Composable BoxScope.() -> Unit, onSettings: () -> Unit) {
    val switcher = remember(browser) { Switcher(browser) }
    val background = MaterialTheme.colorScheme.background
    BackHandler { if (switcher.selection != null) switcher.selection = null else hero.zoomTo(Side.Page) }
    Box(Modifier.fillMaxSize().drawBehind { backdrop(background, hero) }.pointerInput(Unit) {}) {
        Column(Modifier.fillMaxSize().systemBarsPadding()) {
            switcher.selection?.let { SelectBar(it.size, onDone = { switcher.selection = null }, onClose = switcher::closeSelected) }
            Box(Modifier.weight(1f)) {
                Grid(switcher, hero, Modifier.fillMaxSize())
                notices()
            }
            Column(Modifier.graphicsLayer { alpha = hero.dockAlpha; translationY = hero.dockShift }) {
                Toolbar(switcher, onSettings, onDismiss = hero::hide)
                SearchBox(switcher.query, switcher.mode.search, { switcher.query = it }, Modifier.padding(start = 10.dp, end = 10.dp, bottom = 10.dp, top = 2.dp))
            }
        }
        browser.tab?.let { Ghost(it, hero) }
    }
    if (switcher.creating) {
        SpaceDialog(
            browser.spaces.profiles,
            onCreate = { name, profile ->
                browser.addSpace(name, profile)
                switcher.creating = false
                switcher.pick(Mode.Tabs)
            },
            onDismiss = { switcher.creating = false },
        )
    }
    if (switcher.clearing) ConfirmClearBrowsingData(browser, onDismiss = { switcher.clearing = false })
    if (switcher.closingAll) ConfirmCloseAll(switcher)
}
