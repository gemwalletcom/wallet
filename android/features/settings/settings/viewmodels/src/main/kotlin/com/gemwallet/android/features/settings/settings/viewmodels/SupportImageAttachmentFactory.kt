package com.gemwallet.android.features.settings.settings.viewmodels

import android.content.Context
import android.net.Uri
import android.webkit.MimeTypeMap
import com.gemwallet.android.data.repositories.support.ImageAttachment
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SupportImageAttachmentFactory @Inject constructor(
    @ApplicationContext private val context: Context,
) {
    suspend fun fromUri(uri: Uri): ImageAttachment? = withContext(Dispatchers.IO) {
        val bytes = context.contentResolver.openInputStream(uri)?.use { it.readBytes() } ?: return@withContext null
        val mimeType = context.contentResolver.getType(uri) ?: "image/jpeg"
        val extension = MimeTypeMap.getSingleton().getExtensionFromMimeType(mimeType) ?: "jpg"
        ImageAttachment(data = bytes, fileName = "image-${UUID.randomUUID()}.$extension", mimeType = mimeType)
    }
}
