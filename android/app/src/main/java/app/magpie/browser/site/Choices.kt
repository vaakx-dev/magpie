package app.magpie.browser.site

import app.magpie.browser.storage.Retention

data class Choices<T : Any>(val disk: Map<Site, T> = emptyMap(), val memory: Map<Site, T?> = emptyMap()) {
    fun of(site: Site, retention: Retention): T? = when (retention) {
        Retention.Disk -> disk[site]
        Retention.Memory -> if (site in memory) memory[site] else disk[site]
    }

    fun with(site: Site, choice: T?, retention: Retention): Choices<T> = when (retention) {
        Retention.Disk -> copy(disk = if (choice == null) disk - site else disk + (site to choice))
        Retention.Memory -> copy(memory = memory + (site to choice))
    }

    fun sites(retention: Retention): Set<Site> = (disk.keys + memory.keys).filter { of(it, retention) != null }.toSet()

    fun forgetMemory(): Choices<T> = copy(memory = emptyMap())
}
