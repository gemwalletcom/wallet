package com.gemwallet.android.ui.components

import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class QRCodeDecoderTest {
    private val text = "ethereum:0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    private val width = 640
    private val height = 480

    @Test
    fun decodesFrameWhoseRowStrideExceedsWidth() {
        val rowStride = 1024

        assertEquals(text, QRCodeDecoder.decode(lumaFrame(rowStride), rowStride, width, height))
    }

    @Test
    fun decodesFrameWithoutPadding() {
        assertEquals(text, QRCodeDecoder.decode(lumaFrame(width), width, width, height))
    }

    @Test
    fun decodesLightOnDarkFrame() {
        val frame = lumaFrame(width).also { luma -> luma.indices.forEach { luma[it] = (0xFF - (luma[it].toInt() and 0xFF)).toByte() } }

        assertEquals(text, QRCodeDecoder.decode(frame, width, width, height))
    }

    @Test
    fun decodesPixels() {
        val pixels = IntArray(width * height)
        val luma = lumaFrame(width)
        luma.indices.forEach { pixels[it] = if (luma[it] == BLACK) 0xFF000000.toInt() else 0xFFFFFFFF.toInt() }

        assertEquals(text, QRCodeDecoder.decode(pixels, width, height))
    }

    @Test
    fun keepsFullResolutionUpToSixteenMegapixels() {
        assertEquals(1, QRCodeDecoder.sampleSize(1080, 2340))
        assertEquals(1, QRCodeDecoder.sampleSize(1440, 6400))
        assertEquals(1, QRCodeDecoder.sampleSize(4000, 3000))
        assertEquals(2, QRCodeDecoder.sampleSize(8160, 6120))
    }

    @Test
    fun returnsNullWithoutCode() {
        assertNull(QRCodeDecoder.decode(ByteArray(width * height) { WHITE }, width, width, height))
    }

    private fun lumaFrame(rowStride: Int): ByteArray {
        val matrix = QRCodeWriter().encode(text, BarcodeFormat.QR_CODE, 300, 300)
        val frame = ByteArray(rowStride * height) { PADDING }
        for (y in 0 until height) {
            for (x in 0 until width) {
                val inCode = x >= 170 && x < 470 && y >= 90 && y < 390
                frame[y * rowStride + x] = if (inCode && matrix.get(x - 170, y - 90)) BLACK else WHITE
            }
        }
        return frame
    }

    private companion object {
        const val BLACK: Byte = 0x00
        const val WHITE: Byte = 0xFF.toByte()
        const val PADDING: Byte = 0x55
    }
}
