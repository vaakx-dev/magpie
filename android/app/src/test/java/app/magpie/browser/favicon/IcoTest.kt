package app.magpie.browser.favicon

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.ByteBuffer
import java.nio.ByteOrder

private const val RED = 0xFFFF0000.toInt()
private const val GREEN = 0xFF00FF00.toInt()
private const val BLUE = 0xFF0000FF.toInt()
private const val WHITE = 0xFFFFFFFF.toInt()
private const val HALF_RED = 0x80FF0000.toInt()

private class Image(val size: Int, val bitCount: Int, val bytes: ByteArray)

private fun little(size: Int, write: ByteBuffer.() -> Unit): ByteArray =
    ByteBuffer.allocate(size).order(ByteOrder.LITTLE_ENDIAN).apply(write).array()

private fun dib(size: Int, bitCount: Int, rowsBottomUp: List<ByteArray>, maskRowsBottomUp: List<ByteArray>): Image {
    val header = little(40) {
        putInt(40)
        putInt(size)
        putInt(size * 2)
        putShort(1)
        putShort(bitCount.toShort())
    }
    val body = (rowsBottomUp + maskRowsBottomUp).fold(ByteArray(0)) { all, row -> all + row }
    return Image(size, bitCount, header + body)
}

private fun ico(vararg images: Image): ByteArray {
    val directory = 6 + images.size * 16
    val offsets = images.runningFold(directory) { offset, image -> offset + image.bytes.size }
    val head = little(directory) {
        putShort(0)
        putShort(1)
        putShort(images.size.toShort())
        images.forEachIndexed { index, image ->
            put(image.size.toByte())
            put(image.size.toByte())
            put(0)
            put(0)
            putShort(1)
            putShort(image.bitCount.toShort())
            putInt(image.bytes.size)
            putInt(offsets[index])
        }
    }
    return images.fold(head) { all, image -> all + image.bytes }
}

private fun bgra(vararg colors: Int): ByteArray = little(colors.size * 4) { colors.forEach(::putInt) }

private fun mask(vararg bits: Int): ByteArray = byteArrayOf(bits.fold(0) { byte, bit -> byte shl 1 or bit }.shl(8 - bits.size).toByte(), 0, 0, 0)

private fun pixels(bytes: ByteArray): IcoImage.Pixels {
    val image = decodeIco(bytes)
    assertTrue(image is IcoImage.Pixels)
    return image as IcoImage.Pixels
}

class IcoTest {
    @Test
    fun readsTrueColorIconsTopDownWithTheirOwnAlpha() {
        val icon = ico(dib(2, 32, listOf(bgra(BLUE, WHITE), bgra(RED, HALF_RED)), listOf(mask(1, 1), mask(1, 1))))
        val image = pixels(icon)
        assertEquals(2, image.width)
        assertEquals(2, image.height)
        assertArrayEquals(intArrayOf(RED, HALF_RED, BLUE, WHITE), image.argb)
    }

    @Test
    fun cutsOldIconsOutWithTheirTransparencyMask() {
        val bottom = byteArrayOf(0, 0, 0xFF.toByte(), 0, 0xFF.toByte(), 0, 0, 0)
        val top = byteArrayOf(0, 0xFF.toByte(), 0, 0xFF.toByte(), 0xFF.toByte(), 0xFF.toByte(), 0, 0)
        val icon = ico(dib(2, 24, listOf(bottom, top), listOf(mask(0, 1), mask(0, 0))))
        assertArrayEquals(intArrayOf(GREEN, WHITE, RED, 0), pixels(icon).argb)
    }

    @Test
    fun readsPalettedIcons() {
        val palette = bgra(RED, BLUE)
        val header = dib(2, 1, listOf(byteArrayOf(0x40, 0, 0, 0), byteArrayOf(0x80.toByte(), 0, 0, 0)), listOf(mask(0, 0), mask(0, 0)))
        val withPalette = Image(2, 1, header.bytes.copyOf(40) + palette + header.bytes.copyOfRange(40, header.bytes.size))
        assertArrayEquals(intArrayOf(BLUE, RED, RED, BLUE), pixels(ico(withPalette)).argb)
    }

    @Test
    fun picksTheLargestImageAndPassesEmbeddedPngThrough() {
        val png = Image(32, 32, byteArrayOf(0x89.toByte(), 'P'.code.toByte(), 'N'.code.toByte(), 'G'.code.toByte(), 1, 2, 3))
        val small = dib(1, 32, listOf(bgra(RED)), listOf(mask(0)))
        val image = decodeIco(ico(small, png, small))
        assertTrue(image is IcoImage.Png)
        assertArrayEquals(png.bytes, (image as IcoImage.Png).bytes)
    }

    @Test
    fun rejectsFilesThatAreNotWholeIcons() {
        val whole = ico(dib(2, 32, listOf(bgra(RED, RED), bgra(RED, RED)), listOf(mask(0, 0), mask(0, 0))))
        assertNull(decodeIco("<html>404</html>".toByteArray()))
        assertNull(decodeIco(whole.copyOf(whole.size - 20)))
    }
}
