package com.gemwallet.android.ui.components.list_head

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.model.ApprovalValue
import com.gemwallet.android.model.AssetValueHeader
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.R

@Composable
fun AssetValueListHead(header: AssetValueHeader) {
    val amount = when (val value = header.value) {
        is ApprovalValue.Exact -> ValueFormatter(style = ValueFormatter.Style.Full).string(value.value, header.asset)
        ApprovalValue.Unlimited -> stringResource(R.string.simulation_header_unlimited_asset, header.asset.symbol)
    }
    AmountListHead(amount = amount, icon = header.asset)
}
