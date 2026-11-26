package app.magpie.browser.browser

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

class ClosedTab(val space: String, val tab: SavedTab, val index: Int)

const val MAX_CLOSED_TABS = 20

class ClosedTabs {
    private val stack = ArrayDeque<ClosedTab>()
    var announced by mutableStateOf<ClosedTab?>(null)
        private set

    fun push(closed: ClosedTab) {
        stack.addLast(closed)
        if (stack.size > MAX_CLOSED_TABS) stack.removeFirst()
        announced = closed
    }

    fun pop(): ClosedTab? {
        announced = null
        return stack.removeLastOrNull()
    }

    fun dismiss() {
        announced = null
    }

    fun forget(space: String) {
        stack.removeAll { it.space == space }
        if (announced?.space == space) announced = null
    }

    fun clear() {
        stack.clear()
        announced = null
    }
}
