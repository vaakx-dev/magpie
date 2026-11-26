package app.magpie.browser.sponsorblock

import org.junit.Assert.assertEquals
import org.junit.Test

class ViewTest {
    @Test
    fun readsTheCoreSettingsView() {
        val json = """
            {"enabled":true,"show_skip_notice":false,
             "categories":[{"key":"sponsor","label":"Sponsor","description":"Paid promotions","color":"#00d400","on":true},
                           {"key":"filler","label":"Filler","description":"Tangents","color":"#7300ff","on":false}],
             "channels":["UCabc"]}
        """
        assertEquals(
            View(
                enabled = true,
                showSkipNotice = false,
                categories = listOf(
                    CategorySwitch("sponsor", "Sponsor", "Paid promotions", on = true),
                    CategorySwitch("filler", "Filler", "Tangents", on = false),
                ),
                channels = listOf("UCabc"),
            ),
            parseView(json),
        )
    }
}
