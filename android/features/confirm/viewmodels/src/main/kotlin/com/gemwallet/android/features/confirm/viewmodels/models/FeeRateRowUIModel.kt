package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.confirm.FeeRateUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemLocalizedText

data class FeeRateRowUIModel(val model: ListItemModel, val emoji: String, val priority: FeePriority?)

fun FeeRateUIModel.rowUIModel(context: Context): FeeRateRowUIModel = FeeRateRowUIModel(
    model = ListItemModel(
        title = context.getString(priority.stringRes()),
        subtitle = row.value.string(context),
        subtitleExtra = fiatValue.takeIf { it.isNotEmpty() },
    ),
    emoji = emoji,
    priority = priority,
)

fun customFeeRowUIModel(context: Context, customRate: GemLocalizedText?, fiat: String?): FeeRateRowUIModel = FeeRateRowUIModel(
    model = ListItemModel(
        title = context.getString(R.string.fee_rate_custom),
        subtitle = customRate?.string(context),
        subtitleExtra = fiat?.takeIf { it.isNotEmpty() },
    ),
    emoji = "⚙️",
    priority = null,
)
