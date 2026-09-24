package com.gemwallet.android.features.earn.delegation.models

import android.content.Context
import com.gemwallet.android.features.earn.delegation.viewmodels.localization.stringRes
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemDelegationAction
import uniffi.gemstone.GemDelegationDetails
import uniffi.gemstone.GemListRow

class DelegationProperties(val rows: List<DelegationRowUIModel>, val details: GemDelegationDetails, val actions: List<DelegationActionUIModel>, val asset: Asset)

sealed interface DelegationRowUIModel {
    data class Row(val row: GemListRow) : DelegationRowUIModel
    data object Rewards : DelegationRowUIModel
}

data class DelegationActionUIModel(val action: GemDelegationAction, val model: ListItemModel)

internal fun GemDelegationAction.uiModel(context: Context): DelegationActionUIModel = DelegationActionUIModel(
    action = this,
    model = ListItemModel(title = context.getString(stringRes())),
)
