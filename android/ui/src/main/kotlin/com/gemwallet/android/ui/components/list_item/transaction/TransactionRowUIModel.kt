package com.gemwallet.android.ui.components.list_item.transaction

import android.content.Context
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.ui.localization.statusLabelRes
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
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

fun TransactionDataAggregate.uiModel(context: Context) = TransactionRowUIModel(
    title = title.string(context),
    badgeText = if (status.showsBadge) context.getString(state.statusLabelRes()) else null,
    badgeTone = status.tone,
    showsProgress = status.showsProgress,
    subtitle = subtitle.text(context),
    value = value,
    valueTone = valueTone,
    equivalentValue = equivalentValue,
)
