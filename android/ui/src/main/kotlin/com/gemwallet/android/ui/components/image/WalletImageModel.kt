package com.gemwallet.android.ui.components.image

import android.content.Context
import java.io.File

fun walletImageModel(context: Context, imageUrl: String?): Any? =
    if (imageUrl.isNullOrEmpty()) null else File(context.filesDir, imageUrl)
