package com.gemwallet.android.features.earn.delegation.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemDelegationActionItem
import uniffi.gemstone.GemDelegationDestination
import uniffi.gemstone.GemDelegationDetails
import uniffi.gemstone.GemListRow

class DelegationProperties(val rows: List<DelegationRowUIModel>, val details: GemDelegationDetails, val actions: List<DelegationActionUIModel>, val asset: Asset)

sealed interface DelegationRowUIModel {
    data class Row(val row: GemListRow) : DelegationRowUIModel
    data object Rewards : DelegationRowUIModel
}

data class DelegationActionUIModel(val destination: GemDelegationDestination, val model: ListItemModel)

internal fun GemDelegationActionItem.uiModel(context: Context): DelegationActionUIModel = DelegationActionUIModel(
    destination = destination,
    model = ListItemModel(title = context.getString(action.stringRes())),
)
