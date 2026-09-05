package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.wallet.core.primitives.AssetId

@Composable
internal fun AssetHeadItem(
    uiState: AssetInfoUIModel,
    onTransfer: AssetIdAction,
    onReceive: (AssetId) -> Unit,
    onBuy: (AssetId) -> Unit,
    onSwap: (() -> Unit)?,
) {
    AmountListHead(
        amount = uiState.accountInfoUIModel.totalBalance,
        equivalent = uiState.accountInfoUIModel.totalFiat,
        icon = uiState.asset,
    ) {
        AssetHeadActions(
            isViewOnly = uiState.detailsState.isViewOnly,
            buttons = uiState.detailsState.headerButtons,
            onTransfer = { onTransfer(uiState.asset.id) },
            onReceive = { onReceive(uiState.asset.id) },
            onBuy = { onBuy(uiState.asset.id) },
            onSwap = onSwap,
        )
    }
}
