package com.gemwallet.android.ui.components.list_head

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemApprovalValue
import uniffi.gemstone.GemSimulationValue
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R

@Composable
fun AssetValueListHead(header: GemSimulationValue) {
    val amount = when (val value = header.value) {
        is GemApprovalValue.Exact -> ValueFormatter(style = ValueFormatter.Style.Full).string(value.value, header.asset.toPrimitives())
        GemApprovalValue.Unlimited -> stringResource(R.string.simulation_header_unlimited_asset, header.asset.symbol)
    }
    AmountListHead(amount = amount, icon = header.asset.toPrimitives())
}
