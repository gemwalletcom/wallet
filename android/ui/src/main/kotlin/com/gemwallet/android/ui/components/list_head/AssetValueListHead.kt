package com.gemwallet.android.ui.components.list_head

import com.gemwallet.android.ui.localization.string
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemApprovalValue
import uniffi.gemstone.GemSimulationValue
import com.gemwallet.android.model.ValueFormatter
import uniffi.gemstone.GemValueStyle

@Composable
fun AssetValueListHead(header: GemSimulationValue) {
    val amount = header.value.string(
        symbol = header.asset.symbol,
        formatter = ValueFormatter(style = GemValueStyle.FULL),
        asset = header.asset.toPrimitives(),
    )
    AmountListHead(amount = amount, icon = header.asset.toPrimitives())
}
