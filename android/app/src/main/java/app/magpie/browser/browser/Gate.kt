package app.magpie.browser.browser

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

class Gate(private val scope: CoroutineScope, timeoutMs: Long) {
    private val opened = CompletableDeferred<Unit>()

    init {
        scope.launch {
            delay(timeoutMs)
            open()
        }
    }

    fun open() {
        opened.complete(Unit)
    }

    fun then(action: () -> Unit) {
        if (opened.isCompleted) return action()
        scope.launch {
            opened.await()
            action()
        }
    }
}
