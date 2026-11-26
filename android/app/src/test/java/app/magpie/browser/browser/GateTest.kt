package app.magpie.browser.browser

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

private const val TIMEOUT_MS = 1500L

@OptIn(ExperimentalCoroutinesApi::class)
class GateTest {
    @Test
    fun restoredTabWaitsUntilTheExtensionIsReady() = runTest {
        val gate = Gate(backgroundScope, TIMEOUT_MS)
        val woken = mutableListOf<String>()
        gate.then { woken += "restored" }
        runCurrent()
        assertEquals(emptyList<String>(), woken)

        gate.open()
        runCurrent()
        assertEquals(listOf("restored"), woken)

        gate.then { woken += "switched" }
        assertEquals(listOf("restored", "switched"), woken)
    }

    @Test
    fun restoredTabWakesAfterTheTimeoutWhenTheExtensionNeverAnswers() = runTest {
        val gate = Gate(backgroundScope, TIMEOUT_MS)
        val woken = mutableListOf<String>()
        gate.then { woken += "restored" }

        advanceTimeBy(TIMEOUT_MS - 1)
        runCurrent()
        assertEquals(emptyList<String>(), woken)

        advanceTimeBy(1)
        runCurrent()
        assertEquals(listOf("restored"), woken)
    }
}
