package app.magpie.browser.privacy

import app.magpie.browser.core.MagpieCore
import java.io.File

object Filter {
    fun clean(url: String, privacy: Privacy): String =
        if (MagpieCore.loaded) MagpieCore.cleanUrl(url, privacy.stripTracking) else url

    fun cachedSnapshot(dataDir: File): ByteArray? =
        if (MagpieCore.loaded) MagpieCore.cachedFilters(dataDir.absolutePath) else null

    fun refreshedSnapshot(dataDir: File): ByteArray? =
        if (MagpieCore.loaded) MagpieCore.refreshedFilters(dataDir.absolutePath) else null
}
