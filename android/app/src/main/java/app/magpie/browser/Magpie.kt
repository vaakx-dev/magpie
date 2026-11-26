package app.magpie.browser

import android.app.Application
import app.magpie.browser.browser.Browser
import app.magpie.browser.core.MagpieCore

class Magpie : Application() {
    val browser by lazy { Browser(this) }

    override fun onCreate() {
        super.onCreate()
        if (MagpieCore.loaded) MagpieCore.init(filesDir.absolutePath)
    }
}
