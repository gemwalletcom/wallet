package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.theme.paddingLarge
import uniffi.gemstone.GemEmptyStateAction
import uniffi.gemstone.GemEmptyStateKind

@Composable
internal fun EmptyTransactionsItem(size: Int, symbol: String, modifier: Modifier = Modifier, isViewOnly: Boolean = false, onBuy: (() -> Unit)? = null, onSwap: (() -> Unit)? = null) {
    if (size > 0) {
        return
    }
    EmptyContentView(
        type = EmptyContentType(GemEmptyStateKind.ASSET, symbol = symbol, isViewOnly = isViewOnly, actions = mapOf(GemEmptyStateAction.BUY to onBuy, GemEmptyStateAction.SWAP to onSwap)),
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = paddingLarge),
    )
}
