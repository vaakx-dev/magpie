package app.magpie.browser.browser

import org.junit.Assert.assertEquals
import org.junit.Test

class RevivalTest {
    @Test
    fun killedActiveTabReloadsAtOnceAndBackgroundTabWaitsUntilShown() {
        assertEquals(Revive.Now, Revival().ended(1_000L, Presence.Shown))
        assertEquals(Revive.WhenShown, Revival().ended(1_000L, Presence.Hidden))
    }

    @Test
    fun crashLoopStopsReloadingUntilTheTabIsShownAgain() {
        val revival = Revival()
        assertEquals(Revive.Now, revival.ended(1_000L, Presence.Shown))
        assertEquals(Revive.WhenShown, revival.ended(6_000L, Presence.Shown))
        assertEquals(Revive.Now, revival.ended(66_000L, Presence.Shown))
    }
}
