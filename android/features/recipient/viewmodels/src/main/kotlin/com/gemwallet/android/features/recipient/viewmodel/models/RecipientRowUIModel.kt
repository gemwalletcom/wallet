package com.gemwallet.android.features.recipient.viewmodel.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListSection
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemRecipientSection

data class RecipientRowUIModel(val recipient: GemRecipient, val model: ListItemModel)

internal fun GemRecipientSection.uiSection(id: String, context: Context): ListSection<RecipientRowUIModel> = ListSection(
    id = id,
    title = context.getString(kind.stringRes()),
    items = rows.map { row ->
        RecipientRowUIModel(recipient = row.recipient, model = ListItemModel(title = row.title, subtitle = row.subtitle))
    },
)
