package app.magpie.browser.update

import java.io.File
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest

private const val CONNECT_TIMEOUT_MS = 15_000
private const val READ_TIMEOUT_MS = 30_000
private const val BUFFER_BYTES = 64 * 1024
private const val WHOLE_PERCENT = 100
private const val PARTIAL_SUFFIX = ".part"
private val SUCCESS = 200..299

private fun percent(received: Long, size: Long): Int =
    if (size <= 0) WHOLE_PERCENT else (received * WHOLE_PERCENT / size).toInt().coerceAtMost(WHOLE_PERCENT)

private fun copy(input: InputStream, output: OutputStream, digest: MessageDigest, size: Long, onProgress: (Int) -> Unit) {
    val buffer = ByteArray(BUFFER_BYTES)
    var received = 0L
    var reported = -1
    while (true) {
        val read = input.read(buffer)
        if (read < 0) return
        digest.update(buffer, 0, read)
        output.write(buffer, 0, read)
        received += read
        val done = percent(received, size)
        if (done != reported) {
            reported = done
            onProgress(done)
        }
    }
}

private fun open(address: String): HttpURLConnection {
    val connection = URL(address).openConnection() as HttpURLConnection
    connection.connectTimeout = CONNECT_TIMEOUT_MS
    connection.readTimeout = READ_TIMEOUT_MS
    if (connection.responseCode !in SUCCESS) throw IOException("GitHub answered with error ${connection.responseCode}")
    return connection
}

private fun hex(bytes: ByteArray): String = bytes.joinToString("") { "%02x".format(it) }

private fun save(download: Download, file: File, onProgress: (Int) -> Unit): String {
    val digest = MessageDigest.getInstance("SHA-256")
    val connection = open(download.url)
    try {
        connection.inputStream.use { input -> file.outputStream().use { output -> copy(input, output, digest, download.size, onProgress) } }
    } finally {
        connection.disconnect()
    }
    return hex(digest.digest())
}

fun fetch(download: Download, folder: File, onProgress: (Int) -> Unit): File {
    if (File(download.name).name != download.name) throw IOException("The update has an unsafe file name")
    folder.deleteRecursively()
    if (!folder.mkdirs()) throw IOException("Couldn't make room for the update")
    val partial = File(folder, download.name + PARTIAL_SUFFIX)
    val digest = save(download, partial, onProgress)
    if (download.sha256 != null && download.sha256 != digest) {
        partial.delete()
        throw IOException("The download didn't match its checksum, so it wasn't installed")
    }
    val target = File(folder, download.name)
    if (!partial.renameTo(target)) throw IOException("Couldn't save the update")
    return target
}
