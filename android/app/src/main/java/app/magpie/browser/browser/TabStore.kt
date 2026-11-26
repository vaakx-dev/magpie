package app.magpie.browser.browser

import app.magpie.browser.storage.JsonFile

class TabStore(private val file: JsonFile) {
    private var restored = false

    fun restore(): SavedTabs? = file.read(::savedTabs).also { restored = true }

    fun save(tabs: SavedTabs) {
        if (restored) file.write(tabs.json())
    }

    fun flush(tabs: SavedTabs): Boolean {
        save(tabs)
        return file.flush()
    }
}
