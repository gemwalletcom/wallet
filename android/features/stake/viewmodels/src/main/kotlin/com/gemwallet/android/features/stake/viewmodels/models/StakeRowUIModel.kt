package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import uniffi.gemstone.GemStakeActionItem
import uniffi.gemstone.GemStakeActionTap

data class StakeActionUIModel(val tap: GemStakeActionTap, val model: ListItemModel)

internal fun GemStakeActionItem.uiModel(context: Context): StakeActionUIModel = StakeActionUIModel(
    tap = tap,
    model = row.listItemModel(context) ?: ListItemModel(title = ""),
)
