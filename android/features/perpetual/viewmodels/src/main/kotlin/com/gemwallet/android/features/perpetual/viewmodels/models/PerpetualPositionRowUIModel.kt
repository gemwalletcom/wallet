package com.gemwallet.android.features.perpetual.viewmodels.models

import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.wallet.core.primitives.Asset

data class PerpetualPositionRowUIModel(val asset: Asset, val model: ListItemModel)
