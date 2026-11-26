package app.magpie.browser.browser

import org.json.JSONObject
import org.junit.Assert.assertEquals
import org.junit.Test

class ZoomsTest {
    @Test
    fun remembersZoomPerSiteAcrossPagesAndWww() {
        val zooms = SiteZooms().with("https://www.example.com/news", Zoom(130))
        assertEquals(Zoom(130), zooms.of("https://example.com/sport?page=2"))
        assertEquals(Zoom.DEFAULT, zooms.of("https://other.example.org/"))
        assertEquals(Zoom.DEFAULT, zooms.of("about:blank"))
    }

    @Test
    fun resettingTo100PercentForgetsTheSite() {
        val zoomed = SiteZooms().with("https://example.com", Zoom(80)).with("https://magpie.app", Zoom(150))
        val reset = zoomed.with("https://example.com/page", Zoom.DEFAULT)
        assertEquals(mapOf("magpie.app" to Zoom(150)), reset.levels)
    }

    @Test
    fun survivesBeingWrittenAndReadBack() {
        val zooms = SiteZooms().with("https://example.com", Zoom(80)).with("http://localhost:8080/app", Zoom(120))
        val reread = SiteZooms.from(JSONObject(zooms.json().toString()))
        assertEquals(zooms, reread)
        assertEquals(Zoom(120), reread.of("http://localhost:3000/"))
    }
}
