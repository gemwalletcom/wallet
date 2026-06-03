package com.gemwallet.android.features.wallet.viewmodels

import android.graphics.Bitmap
import java.io.ByteArrayOutputStream

internal fun Bitmap.toPng(): ByteArray =
    ByteArrayOutputStream().use { stream ->
        compress(Bitmap.CompressFormat.PNG, 100, stream)
        stream.toByteArray()
    }
