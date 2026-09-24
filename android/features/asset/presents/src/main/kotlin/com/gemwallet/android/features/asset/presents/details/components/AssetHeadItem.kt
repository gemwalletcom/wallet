package com.gemwallet.android.features.asset.presents.details.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.components.list_head.uiModel
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.wallet.core.primitives.AssetId

@Composable
internal fun AssetHeadItem(uiState: AssetInfoUIModel, onTransfer: AssetIdAction, onReceive: (AssetId) -> Unit, onBuy: (AssetId) -> Unit, onSwap: (() -> Unit)?) {
    AmountListHead(
        amount = uiState.details.balanceValue.text(),
        equivalent = uiState.details.fiatValue?.text().orEmpty(),
        icon = uiState.asset,
    ) {
        AssetHeadActions(
            uiState.details.state.headerActions.uiModel(
                onTransfer = { onTransfer(uiState.asset.id) },
                onReceive = { onReceive(uiState.asset.id) },
                onBuy = { onBuy(uiState.asset.id) },
                onSwap = onSwap,
            ),
        )
    }
}
