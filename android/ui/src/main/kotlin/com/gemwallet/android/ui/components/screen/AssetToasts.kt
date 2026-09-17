package com.gemwallet.android.ui.components.screen

import android.content.Context
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage

fun assetPinnedToast(context: Context, name: String, pinned: Boolean): ToastMessage = if (pinned) {
    ToastMessage(title = context.getString(R.string.common_pinned_asset, name), image = R.drawable.ic_push_pin)
} else {
    ToastMessage(title = context.getString(R.string.common_unpinned_asset, name), image = R.drawable.keep_off)
}

fun assetAddedToast(context: Context): ToastMessage = ToastMessage(
    title = context.getString(R.string.asset_added_to_wallet),
    image = R.drawable.ic_add_circle_outlined,
)
