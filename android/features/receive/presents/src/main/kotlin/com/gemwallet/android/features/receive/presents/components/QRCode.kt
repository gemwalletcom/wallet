package com.gemwallet.android.features.receive.presents.components

import android.graphics.Bitmap
import android.graphics.Color
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.painter.BitmapPainter
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.core.graphics.createBitmap
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.qrcode.QRCodeWriter

@Composable
fun rememberQRCodePainter(
    content: String,
    size: Dp = 150.dp,
    padding: Dp = 0.dp,
): BitmapPainter {
    val density = LocalDensity.current
    val sizePx = with(density) { size.roundToPx() }
    val paddingPx = with(density) { padding.roundToPx() }

    return remember(content, sizePx, paddingPx) {
        BitmapPainter(generateQr(content, sizePx, paddingPx).asImageBitmap())
    }
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
