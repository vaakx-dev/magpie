package app.magpie.browser.update

import org.junit.Assert.assertEquals
import org.junit.Test

class ReleaseTest {
    private val page = "https://github.com/vaakx-dev/magpie/releases/tag/v2.0.0"
    private val apk = "https://github.com/vaakx-dev/magpie/releases/download/v2.0.0/magpie-arm64-v8a.apk"

    @Test
    fun readsAnOfferedApkWithOrWithoutAChecksum() {
        val reply = """{"state":"available","version":"2.0.0","page":"$page","notes":"Faster tabs",""" +
            """"download":{"name":"magpie-arm64-v8a.apk","url":"$apk","size":1024,"sha256":%s,"package":"apk"}}"""
        val offer = { sha256: String? -> Phase.Available(Offer("2.0.0", page, "Faster tabs", Download("magpie-arm64-v8a.apk", apk, 1024, sha256))) }

        assertEquals(offer(null), parseStatus(reply.format("null")))
        assertEquals(offer("ab12"), parseStatus(reply.format("\"ab12\"")))
    }

    @Test
    fun readsReleasesWithoutAnApkAndOtherStatuses() {
        assertEquals(
            Phase.Available(Offer("2.0.0", page, "", null)),
            parseStatus("""{"state":"available","version":"2.0.0","page":"$page","notes":"","download":null}"""),
        )
        assertEquals(Phase.UpToDate, parseStatus("""{"state":"up_to_date"}"""))
        assertEquals(
            Phase.Failed("GitHub is busy. Try again later"),
            parseStatus("""{"state":"failed","message":"GitHub is busy. Try again later"}"""),
        )
        assertEquals(Phase.Failed("The update check gave an unreadable answer"), parseStatus("""{"state":"exploded"}"""))
    }
}
