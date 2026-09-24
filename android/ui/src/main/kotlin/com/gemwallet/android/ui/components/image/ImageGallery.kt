package com.gemwallet.android.ui.components.image

import android.content.ContentValues
import android.content.Context
import android.graphics.Bitmap
import android.os.Build
import android.provider.MediaStore
import coil3.SingletonImageLoader
import coil3.request.ImageRequest
import coil3.request.SuccessResult
import coil3.request.allowHardware
import coil3.toBitmap

val canSaveImageToGallery: Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q

suspend fun Context.saveImageToGallery(url: String, name: String) {
    check(canSaveImageToGallery) { "Gallery write needs storage permission below Android 10" }
    val request = ImageRequest.Builder(this).data(url).allowHardware(false).build()
    val result = SingletonImageLoader.get(this).execute(request)
    val bitmap = (result as? SuccessResult)?.image?.toBitmap() ?: error("Image unavailable")
    val values = ContentValues().apply {
        put(MediaStore.Images.Media.DISPLAY_NAME, "$name.png")
        put(MediaStore.Images.Media.MIME_TYPE, "image/png")
    }
    val uri = contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values) ?: error("Gallery insert failed")
    val isWritten = runCatching { contentResolver.openOutputStream(uri)?.use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) } == true }.getOrDefault(false)
    if (!isWritten) {
        contentResolver.delete(uri, null, null)
        error("Gallery write failed")
    }
}
