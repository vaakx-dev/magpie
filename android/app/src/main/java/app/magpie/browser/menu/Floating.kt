package app.magpie.browser.menu

import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntRect
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupPositionProvider
import androidx.compose.ui.window.PopupProperties
import app.magpie.browser.settings.MenuSide
import app.magpie.browser.theme.CARD_SHAPE

private val WIDTH = 260.dp
private val GAP = 6.dp

private class AboveAnchor(private val side: MenuSide, private val gapPx: Int, private val topPx: Int) : PopupPositionProvider {
    var roomPx by mutableIntStateOf(Int.MAX_VALUE)
        private set

    override fun calculatePosition(
        anchorBounds: IntRect,
        windowSize: IntSize,
        layoutDirection: LayoutDirection,
        popupContentSize: IntSize,
    ): IntOffset {
        val bottom = anchorBounds.top - gapPx
        roomPx = bottom - topPx
        val x = when (side) {
            MenuSide.Left -> anchorBounds.left
            MenuSide.Right -> anchorBounds.right - popupContentSize.width
        }
        return IntOffset(x, maxOf(topPx, bottom - popupContentSize.height))
    }
}

private fun Modifier.onBack(action: () -> Unit): Modifier = onPreviewKeyEvent { event ->
    if (event.key != Key.Back) return@onPreviewKeyEvent false
    if (event.type == KeyEventType.KeyUp) action()
    true
}

@Composable
fun Floating(side: MenuSide, onDismiss: () -> Unit, onBack: () -> Unit = onDismiss, content: @Composable ColumnScope.() -> Unit) {
    val density = LocalDensity.current
    val topPx = WindowInsets.statusBars.getTop(density)
    val position = remember(side, density, topPx) { with(density) { AboveAnchor(side, GAP.roundToPx(), topPx) } }
    val maxHeight = with(density) { position.roomPx.toDp() }
    Popup(
        popupPositionProvider = position,
        onDismissRequest = onDismiss,
        properties = PopupProperties(focusable = true, dismissOnBackPress = false),
    ) {
        val focus = remember { FocusRequester() }
        LaunchedEffect(focus) { focus.requestFocus() }
        Surface(
            shape = CARD_SHAPE,
            color = MaterialTheme.colorScheme.surface,
            contentColor = MaterialTheme.colorScheme.onSurface,
            shadowElevation = 10.dp,
            modifier = Modifier.width(WIDTH).heightIn(max = maxHeight).focusRequester(focus).onBack(onBack).focusable(),
        ) {
            Column(Modifier.verticalScroll(rememberScrollState()).padding(PANEL_PADDING), content = content)
        }
    }
}
