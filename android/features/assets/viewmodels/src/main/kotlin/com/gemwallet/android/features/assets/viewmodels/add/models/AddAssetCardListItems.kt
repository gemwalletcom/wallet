package com.gemwallet.android.features.assets.viewmodels.add.models

import android.content.Context
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.theme.Emoji

internal fun verificationWarningListItem(context: Context): ListItemModel = ListItemModel(
    title = context.getString(R.string.asset_verification_warning_title),
    titleExtra = context.getString(R.string.asset_verification_warning_message),
    image = ListItemImage.Emoji(Emoji.warning),
)
