package app.magpie.browser.address

import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChange
import androidx.compose.ui.input.pointer.util.VelocityTracker
import androidx.compose.ui.input.pointer.util.addPointerInputChange
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import kotlin.math.abs

enum class Axis { Across, Up }

val SWIPE_TAB_DISTANCE = 60.dp

class Swipe {
    var axis by mutableStateOf<Axis?>(null)
        private set
    var offset by mutableStateOf(Offset.Zero)
        private set

    fun move(total: Offset) {
        axis = axis ?: if (abs(total.x) > abs(total.y)) Axis.Across else Axis.Up
        offset = total
    }

    fun reset() {
        axis = null
        offset = Offset.Zero
    }
}

class Gesture(val onTap: () -> Unit, val onMove: (Offset) -> Unit, val onRelease: (Offset, Velocity) -> Unit)

fun Modifier.gesture(current: State<Gesture>): Modifier = pointerInput(Unit) {
    awaitEachGesture {
        val gesture by current
        val down = awaitFirstDown()
        val tracker = VelocityTracker().apply { addPointerInputChange(down) }
        var total = Offset.Zero
        var dragging = false
        while (true) {
            val change = awaitPointerEvent().changes.find { it.id == down.id }
            if (change == null) {
                gesture.onRelease(Offset.Zero, Velocity.Zero)
                break
            }
            tracker.addPointerInputChange(change)
            if (!change.pressed) {
                if (dragging) gesture.onRelease(total, tracker.calculateVelocity()) else gesture.onTap()
                break
            }
            total += change.positionChange()
            dragging = dragging || total.getDistance() > viewConfiguration.touchSlop
            if (dragging) {
                change.consume()
                gesture.onMove(total)
            }
        }
    }
}
