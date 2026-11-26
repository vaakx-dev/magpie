package app.magpie.browser.privacy

import app.magpie.browser.site.Site
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PrivacyTest {
    private val news = checkNotNull(Site.of("https://www.news.example/story?id=1"))
    private val shop = checkNotNull(Site.of("https://shop.example/"))

    @Test
    fun pausingTheShieldOnOneSiteKeepsBlockingEverywhereElse() {
        val paused = Privacy().toggle(news)
        assertFalse(paused.blocksAdsOn(news))
        assertFalse(paused.blocksAdsOn(checkNotNull(Site.of("http://news.example/other"))))
        assertTrue(paused.blocksAdsOn(shop))
        assertTrue(paused.toggle(news).blocksAdsOn(news))
    }

    @Test
    fun turningTheAdBlockerOffStopsItOnEverySite() {
        val off = Privacy(adBlock = false)
        assertFalse(off.blocksAdsOn(shop))
        assertFalse(off.resume(shop).blocksAdsOn(shop))
    }
}
