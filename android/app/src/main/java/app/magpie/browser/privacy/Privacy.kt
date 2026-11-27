package app.magpie.browser.privacy

import app.magpie.browser.site.Choices
import app.magpie.browser.site.Site
import app.magpie.browser.storage.Retention

data class Privacy(
    val adBlock: Boolean = true,
    val stripTracking: Boolean = true,
    val trackingProtection: Boolean = true,
    val httpsOnly: Boolean = false,
    val globalPrivacyControl: Boolean = true,
    val clearOnExit: Boolean = false,
    val pausedSites: Choices<Unit> = Choices(),
) {
    fun paused(site: Site, retention: Retention): Boolean = pausedSites.of(site, retention) != null

    fun blocksAdsOn(site: Site, retention: Retention): Boolean = adBlock && !paused(site, retention)

    fun toggle(site: Site, retention: Retention): Privacy =
        if (paused(site, retention)) resume(site, retention) else copy(pausedSites = pausedSites.with(site, Unit, retention))

    fun resume(site: Site, retention: Retention): Privacy = copy(pausedSites = pausedSites.with(site, null, retention))

    fun forgetPrivate(): Privacy = copy(pausedSites = pausedSites.forgetMemory())
}
