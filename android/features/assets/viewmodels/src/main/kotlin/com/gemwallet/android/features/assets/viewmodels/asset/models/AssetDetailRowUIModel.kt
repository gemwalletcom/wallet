package com.gemwallet.android.features.assets.viewmodels.asset.models

import com.gemwallet.android.ui.components.list_item.ListItemModel
import uniffi.gemstone.GemListRow

sealed interface AssetDetailRowUIModel {
    data class Balance(val model: ListItemModel, val action: AssetAction?) : AssetDetailRowUIModel
    data class Row(val row: GemListRow, val action: AssetAction?) : AssetDetailRowUIModel
}
