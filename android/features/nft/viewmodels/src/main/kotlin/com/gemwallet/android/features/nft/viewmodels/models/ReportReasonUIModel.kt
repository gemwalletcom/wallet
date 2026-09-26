package com.gemwallet.android.features.nft.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.ReportReason

data class ReportReasonUIModel(val reason: ReportReason, val model: ListItemModel)

internal fun ReportReason.uiModel(context: Context): ReportReasonUIModel = ReportReasonUIModel(
    reason = this,
    model = ListItemModel(title = context.getString(stringRes())),
)
