package app.magpie.browser.privacy

import app.magpie.browser.site.Site
import app.magpie.browser.storage.Retention
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PrivacyTest {
    private val news = checkNotNull(Site.of("https://www.news.example/story?id=1"))
    private val shop = checkNotNull(Site.of("https://shop.example/"))

    @Test
    fun pausingTheShieldOnOneSiteKeepsBlockingEverywhereElse() {
        val paused = Privacy().toggle(news, Retention.Disk)
        assertFalse(paused.blocksAdsOn(news, Retention.Disk))
        assertFalse(paused.blocksAdsOn(checkNotNull(Site.of("http://news.example/other")), Retention.Disk))
        assertTrue(paused.blocksAdsOn(shop, Retention.Disk))
        assertTrue(paused.toggle(news, Retention.Disk).blocksAdsOn(news, Retention.Disk))
    }

    @Test
    fun aPrivateTabCanResumeASavedPauseWithoutChangingIt() {
        val resumed = Privacy().toggle(news, Retention.Disk).toggle(news, Retention.Memory)
        assertTrue(resumed.blocksAdsOn(news, Retention.Memory))
        assertFalse(resumed.blocksAdsOn(news, Retention.Disk))
        assertFalse(resumed.forgetPrivate().blocksAdsOn(news, Retention.Memory))
    }

    @Test
    fun turningTheAdBlockerOffStopsItOnEverySite() {
        val off = Privacy(adBlock = false)
        assertFalse(off.blocksAdsOn(shop, Retention.Disk))
        assertFalse(off.resume(shop, Retention.Disk).blocksAdsOn(shop, Retention.Disk))
    }
}
