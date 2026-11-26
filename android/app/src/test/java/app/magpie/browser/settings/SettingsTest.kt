package app.magpie.browser.settings

import app.magpie.browser.privacy.Privacy
import app.magpie.browser.site.Site
import org.junit.Assert.assertEquals
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
        ).toggle(Site("news.example")).toggle(Site("shop.example"))
        Settings(preferences).privacy = chosen
        assertEquals(chosen, Settings(preferences).privacy)
    }

    @Test
    fun restoringABackupReloadsPrivacyAndPerSiteChoices() {
        val source = Settings(MemoryPreferences())
        source.privacy = Privacy(httpsOnly = true, clearOnExit = true).toggle(Site("news.example"))
        source.forceDark = source.forceDark.toggle(Site("docs.example"))
        val target = Settings(MemoryPreferences())
        target.restore(source.saved())
        assertEquals(source.privacy, target.privacy)
        assertEquals(source.forceDark, target.forceDark)
    }
}
