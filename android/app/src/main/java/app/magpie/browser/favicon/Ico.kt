package app.magpie.browser.favicon

import java.nio.ByteBuffer
import java.nio.ByteOrder

sealed interface IcoImage {
    class Png(val bytes: ByteArray) : IcoImage

    class Pixels(val width: Int, val height: Int, val argb: IntArray) : IcoImage
}

private class Entry(val width: Int, val bitCount: Int, val size: Int, val offset: Int)

private class Dib(val width: Int, val height: Int, val bitCount: Int, val palette: IntArray, val pixelStart: Int)

private const val ICON_TYPE = 1
private const val DIRECTORY_SIZE = 6
private const val ENTRY_SIZE = 16
private const val FULL_SIZE = 256
private const val DIB_HEADER_SIZE = 40
private const val ROW_ALIGN_BITS = 32
private const val COLORS_USED_AT = 32
private const val MAX_PALETTE_BITS = 8
private const val OPAQUE = 0xFF000000.toInt()
private const val RGB = 0xFFFFFF
private val PNG_SIGNATURE = byteArrayOf(0x89.toByte(), 'P'.code.toByte(), 'N'.code.toByte(), 'G'.code.toByte())
private val SUPPORTED_BIT_COUNTS = setOf(1, 4, 8, 24, 32)

private fun ByteArray.buffer(): ByteBuffer = ByteBuffer.wrap(this).order(ByteOrder.LITTLE_ENDIAN)

private fun ByteBuffer.u8(at: Int): Int = get(at).toInt() and 0xFF

private fun ByteBuffer.u16(at: Int): Int = getShort(at).toInt() and 0xFFFF

private fun dimension(stored: Int): Int = if (stored == 0) FULL_SIZE else stored

private fun stride(width: Int, bitCount: Int): Int = (width * bitCount + ROW_ALIGN_BITS - 1) / ROW_ALIGN_BITS * 4

private fun ico(bytes: ByteArray): Boolean = bytes.size >= DIRECTORY_SIZE && bytes.buffer().run { u16(0) == 0 && u16(2) == ICON_TYPE }

private fun entries(data: ByteBuffer): List<Entry> {
    val count = data.u16(4)
    if (data.capacity() < DIRECTORY_SIZE + count * ENTRY_SIZE) return emptyList()
    return (0 until count).map { DIRECTORY_SIZE + it * ENTRY_SIZE }.map { at ->
        Entry(dimension(data.u8(at)), data.u16(at + 6), data.getInt(at + 8), data.getInt(at + 12))
    }
}

private fun image(bytes: ByteArray, entry: Entry): ByteArray? {
    val end = entry.offset.toLong() + entry.size
    if (entry.offset < 0 || entry.size <= 0 || end > bytes.size) return null
    return bytes.copyOfRange(entry.offset, end.toInt())
}

private fun png(image: ByteArray): Boolean = image.size >= PNG_SIGNATURE.size && image.copyOf(PNG_SIGNATURE.size).contentEquals(PNG_SIGNATURE)

private fun paletteSize(data: ByteBuffer, bitCount: Int): Int {
    if (bitCount > MAX_PALETTE_BITS) return 0
    val full = 1 shl bitCount
    return data.getInt(COLORS_USED_AT).takeIf { it in 1..full } ?: full
}

private fun palette(data: ByteBuffer, headerSize: Int, size: Int): IntArray =
    IntArray(size) { OPAQUE or (data.getInt(headerSize + it * 4) and RGB) }

private fun dib(data: ByteBuffer): Dib? {
    if (data.capacity() < DIB_HEADER_SIZE) return null
    val headerSize = data.getInt(0)
    val width = data.getInt(4)
    val height = data.getInt(8) / 2
    val bitCount = data.u16(14)
    if (headerSize !in DIB_HEADER_SIZE..data.capacity()) return null
    if (width !in 1..FULL_SIZE || height !in 1..FULL_SIZE || bitCount !in SUPPORTED_BIT_COUNTS) return null
    val paletteSize = paletteSize(data, bitCount)
    val pixelStart = headerSize + paletteSize * 4
    if (data.capacity() < pixelStart + stride(width, bitCount) * height) return null
    return Dib(width, height, bitCount, palette(data, headerSize, paletteSize), pixelStart)
}

private fun Dib.color(data: ByteBuffer, row: Int, x: Int): Int {
    val at = pixelStart + (height - 1 - row) * stride(width, bitCount)
    return when (bitCount) {
        32 -> data.getInt(at + x * 4)
        24 -> OPAQUE or (data.u8(at + x * 3 + 2) shl 16) or (data.u8(at + x * 3 + 1) shl 8) or data.u8(at + x * 3)
        else -> {
            val bit = x * bitCount
            val index = (data.u8(at + bit / 8) shr (8 - bitCount - bit % 8)) and ((1 shl bitCount) - 1)
            palette.getOrElse(index) { OPAQUE }
        }
    }
}

private fun Dib.masked(data: ByteBuffer, row: Int, x: Int): Boolean {
    val maskStart = pixelStart + stride(width, bitCount) * height
    val at = maskStart + (height - 1 - row) * stride(width, 1) + x / 8
    return at < data.capacity() && (data.u8(at) shr (7 - x % 8)) and 1 == 1
}

private fun Dib.pixels(data: ByteBuffer): IcoImage.Pixels {
    val colors = IntArray(width * height) { color(data, it / width, it % width) }
    val alphaInPixels = bitCount == 32 && colors.any { it ushr 24 != 0 }
    val argb = if (alphaInPixels) colors else IntArray(colors.size) { if (masked(data, it / width, it % width)) 0 else OPAQUE or colors[it] }
    return IcoImage.Pixels(width, height, argb)
}

fun decodeIco(bytes: ByteArray): IcoImage? {
    if (!ico(bytes)) return null
    val best = entries(bytes.buffer()).maxWithOrNull(compareBy(Entry::width, Entry::bitCount)) ?: return null
    val image = image(bytes, best) ?: return null
    if (png(image)) return IcoImage.Png(image)
    return dib(image.buffer())?.pixels(image.buffer())
}
