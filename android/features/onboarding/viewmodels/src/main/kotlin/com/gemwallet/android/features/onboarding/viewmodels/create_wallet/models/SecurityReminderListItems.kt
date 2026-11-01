package com.gemwallet.android.features.onboarding.viewmodels.create_wallet.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.subtitleRes
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.style.emoji
import uniffi.gemstone.GemSecurityReminderKind
import uniffi.gemstone.securityReminderItems

fun securityReminderListItems(context: Context, kind: GemSecurityReminderKind): List<ListItemModel> = securityReminderItems(kind).map { item ->
    ListItemModel(
        title = context.getString(item.titleRes()),
        titleExtra = context.getString(item.subtitleRes()),
        image = ListItemImage.Emoji(item.emoji()),
    )
}
