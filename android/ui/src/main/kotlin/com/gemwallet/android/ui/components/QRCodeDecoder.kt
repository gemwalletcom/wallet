package com.gemwallet.android.ui.components

import com.google.zxing.BarcodeFormat
import com.google.zxing.BinaryBitmap
import com.google.zxing.DecodeHintType
import com.google.zxing.LuminanceSource
import com.google.zxing.MultiFormatReader
import com.google.zxing.PlanarYUVLuminanceSource
import com.google.zxing.RGBLuminanceSource
import com.google.zxing.common.HybridBinarizer

object QRCodeDecoder {
    private val hints = mapOf(
        DecodeHintType.POSSIBLE_FORMATS to listOf(BarcodeFormat.QR_CODE),
        DecodeHintType.TRY_HARDER to true,
    )

    fun decode(luma: ByteArray, rowStride: Int, width: Int, height: Int): String? =
        decode(PlanarYUVLuminanceSource(luma, rowStride, height, 0, 0, width, height, false))

    fun decode(pixels: IntArray, width: Int, height: Int): String? =
        decode(RGBLuminanceSource(width, height, pixels))

    private fun decode(source: LuminanceSource): String? =
        tryDecode(source) ?: tryDecode(source.invert())

    private fun tryDecode(source: LuminanceSource): String? = try {
        MultiFormatReader().decode(BinaryBitmap(HybridBinarizer(source)), hints).text
    } catch (_: Exception) {
        null
    }
}
