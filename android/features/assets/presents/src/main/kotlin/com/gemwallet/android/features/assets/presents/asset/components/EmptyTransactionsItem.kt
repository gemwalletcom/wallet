package com.gemwallet.android.features.assets.presents.asset.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.theme.paddingLarge
import uniffi.gemstone.GemEmptyState
import uniffi.gemstone.GemEmptyStateAction

@Composable
internal fun EmptyTransactionsItem(size: Int, symbol: String, state: GemEmptyState, onAction: (GemEmptyStateAction) -> Unit, modifier: Modifier = Modifier) {
    if (size > 0) {
        return
    }
    EmptyContentView(
        state = state,
        symbol = symbol,
        onAction = onAction,
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = paddingLarge),
    )
}
