package com.gemwallet.android.features.receive.presents.components

import android.graphics.Bitmap
import android.graphics.Color
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.core.graphics.createBitmap
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.qrcode.QRCodeWriter
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import com.gemwallet.android.ui.theme.space0

private val defaultQrCodeSize = 150.dp

@Composable
fun rememberQRCodePainter(
    content: String,
    size: Dp = defaultQrCodeSize,
    padding: Dp = space0,
): BitmapPainter? {
    val density = LocalDensity.current
    val sizePx = with(density) { size.roundToPx() }
    val paddingPx = with(density) { padding.roundToPx() }

    val painter by produceState<BitmapPainter?>(initialValue = null, content, sizePx, paddingPx) {
        value = withContext(Dispatchers.Default) { BitmapPainter(generateQr(content, sizePx, paddingPx).asImageBitmap()) }
    }
    return painter
}

private fun generateQr(content: String, sizePx: Int, paddingPx: Int): Bitmap {
    val matrix = runCatching {
        QRCodeWriter().encode(content, BarcodeFormat.QR_CODE, sizePx, sizePx, mapOf(EncodeHintType.MARGIN to paddingPx))
    }.getOrNull() ?: return createBitmap(sizePx, sizePx, Bitmap.Config.RGB_565).apply { eraseColor(Color.WHITE) }

    val width = matrix.width
    val height = matrix.height
    val pixels = IntArray(width * height)
    for (y in 0 until height) {
        val row = y * width
        for (x in 0 until width) {
            pixels[row + x] = if (matrix.get(x, y)) Color.BLACK else Color.WHITE
        }
    }
    return Bitmap.createBitmap(pixels, width, height, Bitmap.Config.RGB_565)
}
