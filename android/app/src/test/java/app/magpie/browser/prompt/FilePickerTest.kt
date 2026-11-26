package app.magpie.browser.prompt

import org.junit.Assert.assertArrayEquals
import org.junit.Test

class FilePickerTest {
    @Test
    fun offersOnlyMediaTypesTheSiteAccepts() {
        assertArrayEquals(arrayOf("image/*", "application/pdf"), documentTypes(arrayOf("image/*", " Application/PDF", ".pdf", "image/*")))
    }

    @Test
    fun opensEveryFileWhenTheSiteNamesNoMediaType() {
        assertArrayEquals(arrayOf("*/*"), documentTypes(null))
        assertArrayEquals(arrayOf("*/*"), documentTypes(arrayOf(".docx", "")))
    }
}
