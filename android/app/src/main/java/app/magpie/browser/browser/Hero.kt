package app.magpie.browser.browser

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.tween
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.lerp
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.toSize
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import org.mozilla.geckoview.GeckoView
import kotlin.math.abs
import kotlin.math.roundToInt

enum class Side(val progress: Float) { Page(0f), Tabs(1f) }

private const val ZOOM_MS = 140
private const val MIN_ZOOM_FRACTION = 0.45f
private const val PULL_COMMIT = 0.3f
private val EXPO_OUT = CubicBezierEasing(0.16f, 1f, 0.3f, 1f)
private val PULL_RANGE = 320.dp
private val FLING_PER_SECOND = 500.dp
private val CHROME_SHIFT = 8.dp

private fun firstHalf(progress: Float): Float = (progress * 2).coerceIn(0f, 1f)

private fun secondHalf(progress: Float): Float = (progress * 2 - 1).coerceIn(0f, 1f)

private fun durationMs(from: Float, side: Side): Int = (ZOOM_MS * maxOf(MIN_ZOOM_FRACTION, abs(side.progress - from))).roundToInt()

fun LayoutCoordinates.bounds(): Rect = Rect(positionInRoot(), size.toSize())

class Hero(private val browser: Browser, private val scope: CoroutineScope, density: Density) {
    private val motion = Animatable(Side.Page.progress)
    private var pulled by mutableStateOf<Float?>(null)
    private val cards = mutableStateMapOf<Long, Rect>()
    private val pullRangePx = with(density) { PULL_RANGE.toPx() }
    private val flingPxPerSecond = with(density) { FLING_PER_SECOND.toPx() }
    private val shiftPx = with(density) { CHROME_SHIFT.toPx() }
    var view: GeckoView? = null
    var page by mutableStateOf(Rect.Zero)

    val progress: Float get() = pulled ?: motion.value
    val shown by derivedStateOf { progress > Side.Page.progress || motion.targetValue > Side.Page.progress }
    val flying: Boolean get() = progress < Side.Tabs.progress
    val barAlpha: Float get() = 1f - firstHalf(progress)
    val barShift: Float get() = shiftPx * progress
    val dockAlpha: Float get() = secondHalf(progress)
    val dockShift: Float get() = shiftPx * (1f - progress)

    fun place(tab: Tab, bounds: Rect) {
        cards[tab.id] = bounds
    }

    fun forget(tab: Tab) {
        cards.remove(tab.id)
    }

    fun flight(tab: Tab): Rect = lerp(page, cards[tab.id] ?: page, progress)

    fun open() {
        scope.launch {
            capture()
            animate(Side.Tabs)
        }
    }

    fun enter(tab: Tab) {
        browser.select(tab)
        zoomTo(Side.Page)
    }

    fun zoomTo(side: Side) {
        scope.launch { animate(side) }
    }

    fun hide() {
        pulled = null
        scope.launch { motion.snapTo(Side.Page.progress) }
    }

    fun pull(total: Offset) {
        if (pulled == null) scope.launch { capture() }
        pulled = fraction(total)
    }

    fun release(total: Offset, velocity: Velocity) {
        val flung = -velocity.y > flingPxPerSecond
        zoomTo(if (flung || fraction(total) > PULL_COMMIT) Side.Tabs else Side.Page)
    }

    private fun fraction(total: Offset): Float = (-total.y / pullRangePx).coerceIn(0f, 1f)

    private suspend fun capture() {
        view?.let { browser.tab?.capture(it) }
    }

    private suspend fun animate(side: Side) {
        val from = progress
        motion.snapTo(from)
        pulled = null
        motion.animateTo(side.progress, tween(durationMs(from, side), easing = EXPO_OUT))
    }
}
