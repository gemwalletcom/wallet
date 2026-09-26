package com.gemwallet.android.features.support.viewmodels

import android.content.Context
import android.graphics.Bitmap
import android.graphics.ImageDecoder
import android.net.Uri
import com.gemwallet.android.ext.GemConstants
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SupportImageAttachmentFactory @Inject constructor(@param:ApplicationContext private val context: Context) {
    suspend fun fromUri(uri: Uri): ByteArray? = withContext(Dispatchers.IO) {
        val source = ImageDecoder.createSource(context.contentResolver, uri)
        val bitmap = runCatching {
            ImageDecoder.decodeBitmap(source) { decoder, info, _ ->
                val (width, height) = supportImageSize(info.size.width, info.size.height, GemConstants.supportAttachmentMaxDimension)
                decoder.setTargetSize(width, height)
                decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
            }
        }.getOrNull() ?: return@withContext null
        try {
            ByteArrayOutputStream().use { stream ->
                bitmap.compress(Bitmap.CompressFormat.JPEG, GemConstants.supportAttachmentJpegQuality, stream)
                stream.toByteArray()
            }
        } finally {
            bitmap.recycle()
        }
    }
}

internal fun supportImageSize(width: Int, height: Int, maxDimension: Int): Pair<Int, Int> {
    val longest = maxOf(width, height)
    if (longest <= maxDimension) return width to height
    val scale = maxDimension.toDouble() / longest
    return maxOf(1, (width * scale).toInt()) to maxOf(1, (height * scale).toInt())
}
