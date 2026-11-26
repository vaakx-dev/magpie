package app.magpie.browser.privacy

import app.magpie.browser.site.Site

data class Privacy(
    val adBlock: Boolean = true,
    val stripTracking: Boolean = true,
    val trackingProtection: Boolean = true,
    val httpsOnly: Boolean = false,
    val globalPrivacyControl: Boolean = true,
    val clearOnExit: Boolean = false,
    val pausedSites: Set<Site> = emptySet(),
) {
    fun blocksAdsOn(site: Site): Boolean = adBlock && site !in pausedSites

    fun toggle(site: Site): Privacy = if (site in pausedSites) resume(site) else copy(pausedSites = pausedSites + site)

    fun resume(site: Site): Privacy = copy(pausedSites = pausedSites - site)
}
