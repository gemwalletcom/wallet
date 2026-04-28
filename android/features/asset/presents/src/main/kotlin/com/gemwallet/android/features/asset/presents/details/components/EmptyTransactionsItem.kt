package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.theme.paddingLarge

@Composable
internal fun EmptyTransactionsItem(
    size: Int,
    symbol: String,
    modifier: Modifier = Modifier,
    isViewOnly: Boolean = false,
    onBuy: (() -> Unit)? = null,
    onSwap: (() -> Unit)? = null,
) {
    if (size > 0) {
        return
    }
    EmptyContentView(
        type = EmptyContentType.Asset(symbol = symbol, onBuy = onBuy, onSwap = onSwap, isViewOnly = isViewOnly),
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = paddingLarge),
    )
}
