package app.magpie.browser.menu

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
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

private val SHAPE = RoundedCornerShape(12.dp)
private val WIDTH = 260.dp
private val GAP = 6.dp

private class AboveAnchor(private val side: MenuSide, private val gapPx: Int) : PopupPositionProvider {
    override fun calculatePosition(
        anchorBounds: IntRect,
        windowSize: IntSize,
        layoutDirection: LayoutDirection,
        popupContentSize: IntSize,
    ): IntOffset {
        val x = when (side) {
            MenuSide.Left -> anchorBounds.left
            MenuSide.Right -> anchorBounds.right - popupContentSize.width
        }
        return IntOffset(x, anchorBounds.top - gapPx - popupContentSize.height)
    }
}

@Composable
fun Floating(side: MenuSide, onDismiss: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    val density = LocalDensity.current
    val position = remember(side, density) { with(density) { AboveAnchor(side, GAP.roundToPx()) } }
    Popup(popupPositionProvider = position, onDismissRequest = onDismiss, properties = PopupProperties(focusable = true)) {
        Surface(
            shape = SHAPE,
            color = MaterialTheme.colorScheme.surface,
            contentColor = MaterialTheme.colorScheme.onSurface,
            shadowElevation = 10.dp,
            modifier = Modifier.width(WIDTH),
        ) {
            Column(Modifier.padding(PANEL_PADDING), content = content)
        }
    }
}
