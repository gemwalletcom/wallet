package com.gemwallet.android.ui.components.image

import android.graphics.Bitmap
import java.io.ByteArrayOutputStream

fun Bitmap.toPng(): ByteArray =
    ByteArrayOutputStream().use { stream ->
        compress(Bitmap.CompressFormat.PNG, 100, stream)
        stream.toByteArray()
    }
