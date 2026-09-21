package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.ByteArrayOutputStream
import javax.inject.Inject
import javax.inject.Singleton

private const val SUPPORT_IMAGE_JPEG_QUALITY = 90

@Singleton
class SupportImageAttachmentFactory @Inject constructor(@param:ApplicationContext private val context: Context) {
    suspend fun fromUri(uri: Uri): ByteArray? = withContext(Dispatchers.IO) {
        val bitmap = context.contentResolver.openInputStream(uri)?.use {
            BitmapFactory.decodeStream(it)
        } ?: return@withContext null
        ByteArrayOutputStream().use { stream ->
            bitmap.compress(Bitmap.CompressFormat.JPEG, SUPPORT_IMAGE_JPEG_QUALITY, stream)
            stream.toByteArray()
        }
    }
}
