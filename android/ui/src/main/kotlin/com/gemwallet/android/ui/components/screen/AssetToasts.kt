package com.gemwallet.android.ui.components.screen

import android.content.Context
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.style.iconRes
import uniffi.gemstone.GemToast

fun GemToast.message(context: Context): ToastMessage = ToastMessage(title = text.string(context), image = icon.iconRes())

fun assetAddedToast(context: Context): ToastMessage = ToastMessage(
    title = context.getString(R.string.asset_added_to_wallet),
    image = R.drawable.ic_add_circle_outlined,
)
