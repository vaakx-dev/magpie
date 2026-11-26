package app.magpie.browser.privacy

data class Privacy(
    val adBlock: Boolean = true,
    val stripTracking: Boolean = true,
    val trackingProtection: Boolean = true,
)
