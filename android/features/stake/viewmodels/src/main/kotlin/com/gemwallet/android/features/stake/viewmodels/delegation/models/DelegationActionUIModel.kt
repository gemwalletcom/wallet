package com.gemwallet.android.features.stake.viewmodels.delegation.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import uniffi.gemstone.GemDelegationActionItem
import uniffi.gemstone.GemDelegationDestination

data class DelegationActionUIModel(val destination: GemDelegationDestination, val model: ListItemModel)

internal fun GemDelegationActionItem.uiModel(context: Context): DelegationActionUIModel = DelegationActionUIModel(
    destination = destination,
    model = ListItemModel(title = context.getString(action.stringRes())),
)
