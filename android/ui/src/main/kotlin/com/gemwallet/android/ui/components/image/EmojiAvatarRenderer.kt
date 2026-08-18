package com.gemwallet.android.ui.components.image

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import com.gemwallet.android.ui.theme.extraLargeIconSize
import kotlin.math.roundToInt

object EmojiAvatarRenderer {

    fun render(context: Context, emoji: String, backgroundColor: Int): ByteArray {
        val size = (extraLargeIconSize.value * context.resources.displayMetrics.density).roundToInt()
        val bitmap = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        val center = size / 2f

        val backgroundPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { color = backgroundColor }
        canvas.drawCircle(center, center, center, backgroundPaint)

        val textPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            textSize = size * EMOJI_AVATAR_SCALE
            textAlign = Paint.Align.CENTER
        }
        val metrics = textPaint.fontMetrics
        val baseline = center - (metrics.ascent + metrics.descent) / 2f
        canvas.drawText(emoji, center, baseline, textPaint)

        return bitmap.toPng()
    }
}
