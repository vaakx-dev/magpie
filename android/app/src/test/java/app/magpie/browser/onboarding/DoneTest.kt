package app.magpie.browser.onboarding

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class DoneTest {
    @Test
    fun freshProfileWithNoSavedPreferencesStillNeedsOnboarding() {
        assertFalse(onboarded(emptyMap<String, Any>()))
    }

    @Test
    fun existingProfileFromBeforeOnboardingCountsAsDone() {
        assertTrue(onboarded(mapOf("theme" to "Dark", "favorites" to "[]")))
    }

    @Test
    fun savedFlagWinsOverOtherPreferences() {
        assertFalse(onboarded(mapOf(ONBOARDING_DONE to false, "spaces" to "[]")))
        assertTrue(onboarded(mapOf(ONBOARDING_DONE to true)))
    }
}
