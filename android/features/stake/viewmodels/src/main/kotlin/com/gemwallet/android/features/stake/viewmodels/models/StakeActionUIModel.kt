package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import uniffi.gemstone.GemStakeAction
import uniffi.gemstone.GemStakeActionItem

data class StakeActionUIModel(val action: GemStakeAction, val model: ListItemModel)

internal fun GemStakeActionItem.uiModel(context: Context): StakeActionUIModel = StakeActionUIModel(
    action = action,
    model = row.listItemModel(context) ?: ListItemModel(title = ""),
)
