package com.gemwallet.android.features.stake.viewmodels.models

import android.content.Context
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.listItemModel
import uniffi.gemstone.GemStakeActionItem
import uniffi.gemstone.GemStakeActionTap

data class StakeActionUIModel(val tap: GemStakeActionTap, val model: ListItemModel)

internal fun GemStakeActionItem.uiModel(context: Context, assetInfo: AssetInfo): StakeActionUIModel = StakeActionUIModel(
    tap = tap,
    model = row.listItemModel(context, assetInfo.id().iconModel()) ?: ListItemModel(title = ""),
)
