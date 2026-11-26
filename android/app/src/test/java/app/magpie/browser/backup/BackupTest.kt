package app.magpie.browser.backup

import app.magpie.browser.favorite.Favorite
import app.magpie.browser.settings.Preference
import app.magpie.browser.space.Profile
import app.magpie.browser.space.Space
import org.junit.Assert.assertEquals
import org.junit.Test

class BackupTest {
    private val backup = Backup(
        settings = mapOf(
            "theme" to Preference.Text("Dark"),
            "custom_search" to Preference.Text("https://search.example.org/?q=%s"),
            "ad_block" to Preference.Switch(false),
        ),
        spaces = listOf(
            Space("personal", "Personal", Profile.Named("personal", "Personal")),
            Space("garden", "Garden club", Profile.Named("personal", "Personal")),
        ),
        favorites = listOf(Favorite("https://example.com/", "Example \"quoted\" ✓")),
    )

    private fun problem(text: String): String = (parse(text) as Parsed.Invalid).problem

    @Test
    fun exportsAndImportsTheSameBackup() {
        assertEquals(Parsed.Valid(backup), parse(backup.json()))
        assertEquals(Parsed.Valid(backup), parse("\uFEFF" + backup.json()))
    }

    @Test
    fun explainsFilesItCannotRestore() {
        assertEquals("This file isn't a Magpie backup.", problem("not json"))
        assertEquals("This file isn't a Magpie backup.", problem("""{"app":"other","format":1}"""))
        assertEquals(
            "This backup is from a newer Magpie (format 9). Update Magpie to restore it.",
            problem(backup.json().replace("\"format\": 1", "\"format\": 9")),
        )
        assertEquals(
            "This backup is from Magpie for desktop. Restore it there; Android can't read desktop settings.",
            problem("""{"app":"magpie","format":1,"settings":"theme = 'dark'","sponsorblock":"","session":{}}"""),
        )
        assertEquals(
            "This backup is damaged: setting theme has an unexpected value",
            problem("""{"app":"magpie","format":1,"platform":"android","settings":{"theme":3},"spaces":[],"favorites":[]}"""),
        )
    }
}
