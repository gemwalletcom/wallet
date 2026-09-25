package com.gemwallet.android.features.create_wallet.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.features.create_wallet.viewmodels.localization.subtitleRes
import com.gemwallet.android.features.create_wallet.viewmodels.localization.titleRes
import com.gemwallet.android.features.create_wallet.viewmodels.style.emoji
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel

fun securityReminderListItems(context: Context): List<ListItemModel> = GemConstants.securityReminderItems.map { item ->
    ListItemModel(
        title = context.getString(item.titleRes()),
        titleLineLimit = 2,
        titleExtra = context.getString(item.subtitleRes()),
        image = ListItemImage.Emoji(item.emoji()),
    )
}
