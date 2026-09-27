package com.gemwallet.android.features.wallet_connector.presents

import androidx.compose.runtime.Composable
import com.gemwallet.android.features.wallet_connector.viewmodels.models.SignMessageUIState
import com.gemwallet.android.ui.components.list_item.GemListRowView
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.ChainAddress

@Composable
internal fun SignMessageScene(state: SignMessageUIState, buttonState: ButtonState, onSign: () -> Unit, onReject: () -> Unit, onOpenAddress: (ChainAddress) -> Unit) {
    WalletConnectReviewScene(
        model = state,
        buttonState = buttonState,
        details = { itemsPositioned(state.rows) { position, row -> GemListRowView(row = row, listPosition = position) } },
        onApprove = onSign,
        onReject = onReject,
        onOpenAddress = onOpenAddress,
    )
}
