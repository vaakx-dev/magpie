package app.magpie.browser.update

import org.junit.Assert.assertEquals
import org.junit.Test

class BuildTest {
    @Test
    fun stableBuildsShowTheirVersion() {
        assertEquals("Magpie 0.1.1", versionLabel("0.1.1", ""))
    }

    @Test
    fun nightlyBuildsShowWhenAndFromWhichCommitTheyWereCut() {
        assertEquals(
            "Magpie 0.1.1 (Nightly 2026-11-26 16:00 UTC · abc1234)",
            versionLabel("0.1.1-nightly.202611261600", "abc1234"),
        )
        assertEquals("Magpie 0.1.1 (Nightly 2026-11-26 16:00 UTC)", versionLabel("0.1.1-nightly.202611261600", ""))
    }
}
