package com.gemwallet.android.ui.components.list_head

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemSimulationValue
import uniffi.gemstone.GemValueStyle

data class SimulationHeaderUIModel(val asset: Asset, val amount: String)

fun GemSimulationValue.headerUIModel(context: Context): SimulationHeaderUIModel {
    val asset = this.asset.toPrimitives()
    return SimulationHeaderUIModel(
        asset = asset,
        amount = value.text(context, asset.symbol, ValueFormatter(style = GemValueStyle.FULL), asset),
    )
}
