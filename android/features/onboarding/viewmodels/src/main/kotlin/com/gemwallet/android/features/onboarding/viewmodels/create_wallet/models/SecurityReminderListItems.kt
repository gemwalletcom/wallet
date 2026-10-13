package com.gemwallet.android.features.onboarding.viewmodels.create_wallet.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.subtitleRes
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.style.emoji
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.securityReminderItems

fun securityReminderListItems(context: Context, kind: GemWalletSecretKind): List<ListItemModel> = securityReminderItems(kind).map { item ->
    ListItemModel(
        title = context.getString(item.titleRes()),
        titleLineLimit = 2,
        titleExtra = context.getString(item.subtitleRes()),
        image = ListItemImage.Emoji(item.emoji()),
    )
}
