package app.magpie.browser.permission

import app.magpie.browser.favicon.Origin
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class DecisionsTest {
    private val site = Origin("https://meet.example.com")
    private val other = Origin("https://example.org")
    private val media = setOf(Kind.Camera, Kind.Microphone)

    private fun reloaded(decisions: Decisions): Decisions = Decisions.stored(decisions.encoded())

    @Test
    fun remembersEachSiteSeparatelyAcrossRestarts() {
        val saved = reloaded(Decisions().with(site, media, Decision.Allow).with(other, setOf(Kind.Location), Decision.Block))
        assertEquals(Decision.Allow, saved.decision(site, media))
        assertEquals(Decision.Allow, saved.decision(site, setOf(Kind.Microphone)))
        assertEquals(Decision.Block, saved.decision(other, setOf(Kind.Location)))
        assertNull(saved.decision(other, media))
        assertNull(saved.decision(site, setOf(Kind.Location)))
    }

    @Test
    fun asksAgainUnlessEveryRequestedKindWasAllowed() {
        val cameraOnly = Decisions().with(site, setOf(Kind.Camera), Decision.Allow)
        assertNull(cameraOnly.decision(site, media))
        val micBlocked = cameraOnly.with(site, setOf(Kind.Microphone), Decision.Block)
        assertEquals(Decision.Block, micBlocked.decision(site, media))
    }

    @Test
    fun laterDecisionReplacesEarlierOne() {
        val changed = Decisions().with(site, setOf(Kind.Notifications), Decision.Block).with(site, setOf(Kind.Notifications), Decision.Allow)
        assertEquals(Decision.Allow, reloaded(changed).decision(site, setOf(Kind.Notifications)))
    }

    @Test
    fun startsEmptyAndSkipsUnknownStoredEntries() {
        assertNull(Decisions.stored(null).decision(site, setOf(Kind.Camera)))
        val stored = Decisions.stored("""{"https://meet.example.com":{"Camera":"Allow","Teleport":"Allow","Microphone":"Maybe"}}""")
        assertEquals(Decision.Allow, stored.decision(site, setOf(Kind.Camera)))
        assertNull(stored.decision(site, setOf(Kind.Microphone)))
    }
}
