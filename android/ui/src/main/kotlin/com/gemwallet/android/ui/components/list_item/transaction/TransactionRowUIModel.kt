package com.gemwallet.android.ui.components.list_item.transaction

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.localization.statusLabelRes
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionStateTone
import uniffi.gemstone.GemValueTone

data class TransactionRowUIModel(
    val title: String,
    val badgeText: String?,
    val badgeTone: GemTransactionStateTone,
    val showsProgress: Boolean,
    val subtitle: String?,
    val value: String,
    val valueTone: GemValueTone,
    val equivalentValue: String?,
)

fun GemTransactionRow.uiModel(context: Context) = TransactionRowUIModel(
    title = title.string(context),
    badgeText = if (status.showsBadge) context.getString(state.toPrimitives().statusLabelRes()) else null,
    badgeTone = status.tone,
    showsProgress = status.showsProgress,
    subtitle = subtitle.text(context),
    value = value.text().orEmpty(),
    valueTone = valueTone,
    equivalentValue = equivalentValue.text(),
)
