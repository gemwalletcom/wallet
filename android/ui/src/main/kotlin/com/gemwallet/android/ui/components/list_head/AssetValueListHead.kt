package com.gemwallet.android.ui.components.list_head

import androidx.compose.runtime.Composable

@Composable
fun AssetValueListHead(header: SimulationHeaderUIModel) {
    AmountListHead(amount = header.amount, icon = header.icon)
}
