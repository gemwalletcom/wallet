package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import com.gemwallet.android.data.repositories.support.ImageAttachment
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

private const val SUPPORT_IMAGE_JPEG_QUALITY = 90
private const val SUPPORT_IMAGE_MIME_TYPE = "image/jpeg"

@Singleton
class SupportImageAttachmentFactory @Inject constructor(
    @param:ApplicationContext private val context: Context,
) {
    suspend fun fromUri(uri: Uri): ImageAttachment? = withContext(Dispatchers.IO) {
        val bitmap = context.contentResolver.openInputStream(uri)?.use {
            BitmapFactory.decodeStream(it)
        } ?: return@withContext null
        val jpegBytes = ByteArrayOutputStream().use { stream ->
            bitmap.compress(Bitmap.CompressFormat.JPEG, SUPPORT_IMAGE_JPEG_QUALITY, stream)
            stream.toByteArray()
        }
        ImageAttachment(data = jpegBytes, fileName = "image-${UUID.randomUUID()}.jpg", mimeType = SUPPORT_IMAGE_MIME_TYPE)
    }
}
