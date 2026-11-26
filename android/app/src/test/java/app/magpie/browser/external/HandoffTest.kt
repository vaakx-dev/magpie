package app.magpie.browser.external

import org.junit.Assert.assertEquals
import org.junit.Test

class HandoffTest {
    @Test
    fun opensAppsTheUserAskedFor() {
        assertEquals(Handoff.Open, handoff(Navigation(userGesture = true, typed = false)))
        assertEquals(Handoff.Open, handoff(Navigation(userGesture = false, typed = true)))
    }

    @Test
    fun confirmsAppsAPageOpensOnItsOwn() {
        assertEquals(Handoff.Confirm, handoff(Navigation(userGesture = false, typed = false)))
    }
}
