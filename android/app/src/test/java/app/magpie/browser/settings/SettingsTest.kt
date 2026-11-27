package app.magpie.browser.settings

import app.magpie.browser.privacy.Privacy
import app.magpie.browser.site.Site
import app.magpie.browser.storage.Retention
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SettingsTest {
    @Test
    fun privacyChoicesSurviveARestart() {
        val preferences = MemoryPreferences()
        val chosen = Privacy(
            adBlock = false,
            stripTracking = false,
            trackingProtection = false,
            httpsOnly = true,
            globalPrivacyControl = false,
            clearOnExit = true,
        ).toggle(Site("news.example"), Retention.Disk).toggle(Site("shop.example"), Retention.Disk)
        Settings(preferences).privacy = chosen
        assertEquals(chosen, Settings(preferences).privacy)
    }

    @Test
    fun choicesMadeInPrivateTabsApplyThereButAreNeverSaved() {
        val preferences = MemoryPreferences()
        val docs = Site("docs.example")
        val news = Site("news.example")
        val settings = Settings(preferences)
        settings.forceDark = settings.forceDark.toggle(docs, Retention.Memory)
        settings.privacy = settings.privacy.toggle(news, Retention.Memory)

        assertTrue(settings.forceDark.on(docs, Retention.Memory))
        assertFalse(settings.privacy.blocksAdsOn(news, Retention.Memory))
        assertFalse(settings.forceDark.on(docs, Retention.Disk))
        assertTrue(settings.privacy.blocksAdsOn(news, Retention.Disk))

        val stored = preferences.all.toString()
        assertFalse(stored.contains(docs.name))
        assertFalse(stored.contains(news.name))
        val restarted = Settings(preferences)
        assertFalse(restarted.forceDark.on(docs, Retention.Memory))
        assertTrue(restarted.privacy.blocksAdsOn(news, Retention.Memory))
    }

    @Test
    fun restoringABackupReloadsPrivacyAndPerSiteChoices() {
        val source = Settings(MemoryPreferences())
        source.privacy = Privacy(httpsOnly = true, clearOnExit = true).toggle(Site("news.example"), Retention.Disk)
        source.forceDark = source.forceDark.toggle(Site("docs.example"), Retention.Disk)
        val target = Settings(MemoryPreferences())
        target.restore(source.saved())
        assertEquals(source.privacy, target.privacy)
        assertEquals(source.forceDark, target.forceDark)
    }
}
