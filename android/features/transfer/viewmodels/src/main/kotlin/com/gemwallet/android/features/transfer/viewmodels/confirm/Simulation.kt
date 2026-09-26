package com.gemwallet.android.features.transfer.viewmodels.confirm

import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.style.textStyle
import uniffi.gemstone.GemSimulationBalanceChange

fun GemSimulationBalanceChange.listItem(): ListItemModel = ListItemModel(
    title = asset.name,
    subtitle = amount.text(),
    subtitleStyle = amount.tone.textStyle(),
    image = ListItemImage.Asset(icon),
)
