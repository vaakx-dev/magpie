package app.magpie.browser.download

import org.junit.Assert.assertEquals
import org.junit.Test

class FileNameTest {
    private val page = "https://files.example.com/get?id=7"

    @Test
    fun prefersTheEncodedNameThenTheQuotedOneThenTheBareOne() {
        assertEquals("résumé 2024.pdf", fileName("""attachment; filename="fallback.pdf"; filename*=UTF-8''r%C3%A9sum%C3%A9%202024.pdf""", page, "pdf"))
        assertEquals("report; final.pdf", fileName("""attachment; filename="report; final.pdf"""", page, "pdf"))
        assertEquals("say _hi_.txt", fileName("""attachment; filename="say \"hi\".txt"""", page, "txt"))
        assertEquals("notes.txt", fileName("attachment; filename=notes.txt", page, "txt"))
    }

    @Test
    fun fallsBackToTheLastPathSegmentOfTheUrl() {
        assertEquals("tool v2.zip", fileName("attachment", "https://example.com/releases/tool%20v2.zip?x=1", "zip"))
        assertEquals("a+b.zip", fileName(null, "https://example.com/a+b.zip", "zip"))
    }

    @Test
    fun namesUntitledDownloadsAndAddsTheTypesExtension() {
        assertEquals("download.pdf", fileName(null, "blob:https://example.com/9b1f-44c2", "pdf"))
        assertEquals("download", fileName(null, "https://example.com/", null))
        assertEquals("export.csv", fileName(null, "https://example.com/export", "csv"))
        assertEquals("bashrc.txt", fileName("attachment; filename=.bashrc", page, "txt"))
    }

    @Test
    fun keepsTheNameInsideTheDownloadsFolder() {
        assertEquals("_.._etc_passwd", fileName("""attachment; filename="/../etc/passwd"""", page, null))
        assertEquals("a_b_c.txt", fileName("""attachment; filename="a:b*c.txt"""", page, "txt"))
    }
}
