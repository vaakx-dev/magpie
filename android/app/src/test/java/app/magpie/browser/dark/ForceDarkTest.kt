package app.magpie.browser.dark

import app.magpie.browser.site.Site
import app.magpie.browser.storage.Retention
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ForceDarkTest {
    private val docs = Site("docs.example")
    private val video = Site("video.example")

    @Test
    fun excludedSiteStaysLightWhileEveryOtherSiteIsDarkened() {
        val excluded = ForceDark(enabled = true).toggle(docs, Retention.Disk)
        assertFalse(excluded.on(docs, Retention.Disk))
        assertTrue(excluded.on(video, Retention.Disk))
        assertTrue(excluded.toggle(docs, Retention.Disk).on(docs, Retention.Disk))
        assertFalse(excluded.toggle(docs, Retention.Disk).chosen(docs, Retention.Disk))
    }

    @Test
    fun siteCanBeDarkenedWhileForceDarkIsOff() {
        val darkened = ForceDark(enabled = false).toggle(video, Retention.Disk)
        assertTrue(darkened.on(video, Retention.Disk))
        assertFalse(darkened.on(docs, Retention.Disk))
    }

    @Test
    fun privateTabsCanOverrideASavedChoiceUntilThePrivateSessionEnds() {
        val saved = ForceDark(enabled = true).toggle(docs, Retention.Disk)
        val overridden = saved.toggle(docs, Retention.Memory)
        assertTrue(overridden.on(docs, Retention.Memory))
        assertFalse(overridden.chosen(docs, Retention.Memory))
        assertFalse(overridden.on(docs, Retention.Disk))
        assertFalse(overridden.forgetPrivate().on(docs, Retention.Memory))
    }

    @Test
    fun siteChoicesSurviveARestart() {
        val chosen = ForceDark(enabled = true).toggle(docs, Retention.Disk).copy(enabled = false).toggle(video, Retention.Disk)
        assertEquals(chosen, ForceDark.stored(chosen.enabled, chosen.encodedSites()))
        assertEquals(ForceDark(), ForceDark.stored(false, null))
    }
}
