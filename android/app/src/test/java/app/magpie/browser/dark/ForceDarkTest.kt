package app.magpie.browser.dark

import app.magpie.browser.site.Site
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ForceDarkTest {
    private val docs = Site("docs.example")
    private val video = Site("video.example")

    @Test
    fun excludedSiteStaysLightWhileEveryOtherSiteIsDarkened() {
        val excluded = ForceDark(enabled = true).toggle(docs)
        assertFalse(excluded.on(docs))
        assertTrue(excluded.on(video))
        assertTrue(excluded.toggle(docs).on(docs))
        assertEquals(emptyMap<Site, Boolean>(), excluded.toggle(docs).sites)
    }

    @Test
    fun siteCanBeDarkenedWhileForceDarkIsOff() {
        val darkened = ForceDark(enabled = false).toggle(video)
        assertTrue(darkened.on(video))
        assertFalse(darkened.on(docs))
        assertFalse(darkened.reset(video).on(video))
    }

    @Test
    fun siteChoicesSurviveARestart() {
        val chosen = ForceDark(enabled = true).toggle(docs).copy(enabled = false).toggle(video)
        assertEquals(chosen, ForceDark.stored(chosen.enabled, chosen.encodedSites()))
        assertEquals(ForceDark(), ForceDark.stored(false, null))
    }
}
