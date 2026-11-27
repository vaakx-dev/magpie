package app.magpie.browser.download

import android.net.Uri
import androidx.compose.runtime.mutableStateListOf
import app.magpie.browser.storage.JsonFile
import app.magpie.browser.storage.Retention

private val KEPT = setOf(Progress.Done, Progress.Failed)

private val Download.kept: Boolean get() = progress in KEPT && retention == Retention.Disk

private fun Download.record(): Record = Record(name, type, total, progress, received, uri?.toString())

private fun Record.download(): Download = Download(name, type, total, Retention.Disk).also {
    it.progress = progress
    it.received = received
    it.uri = uri?.let(Uri::parse)
}

class Ledger(private val file: JsonFile) {
    val list = mutableStateListOf<Download>().apply { addAll(file.read(::records).orEmpty().map { it.download() }) }

    fun add(download: Download) {
        list.add(0, download)
    }

    fun finish(download: Download, progress: Progress) {
        download.progress = progress
        persist()
    }

    fun clear() {
        list.removeAll { it.progress != Progress.Running }
        persist()
    }

    fun flush() {
        file.flush()
    }

    private fun persist() = file.write(list.filter { it.kept }.map { it.record() }.json())
}
