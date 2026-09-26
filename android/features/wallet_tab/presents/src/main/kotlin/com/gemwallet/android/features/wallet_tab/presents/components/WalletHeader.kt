package com.gemwallet.android.features.wallet_tab.presents.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.domains.wallet.aggregates.WalletSummary
import com.gemwallet.android.ui.components.HideToggle
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.components.list_head.ValueListHead
import uniffi.gemstone.GemHeaderButtonAction

@Composable
internal fun WalletHeader(walletSummary: WalletSummary?, onSendClick: () -> Unit, onReceiveClick: () -> Unit, onBuyClick: () -> Unit, onSwapClick: () -> Unit, onHideBalances: () -> Unit, onPortfolio: () -> Unit) {
    walletSummary ?: return

    val header = walletSummary.state.header
    ValueListHead(
        header = header,
        hideToggle = HideToggle(
            hidden = walletSummary.isBalanceHidden,
            onToggle = onHideBalances,
        ),
        onSubtitleClick = onPortfolio,
        actions = {
            AssetHeadActions(header.actions ?: return@ValueListHead) { action ->
                when (action) {
                    is GemHeaderButtonAction.Send -> onSendClick()
                    is GemHeaderButtonAction.Receive -> onReceiveClick()
                    is GemHeaderButtonAction.Buy -> onBuyClick()
                    is GemHeaderButtonAction.Swap -> onSwapClick()
                    is GemHeaderButtonAction.Deposit, is GemHeaderButtonAction.Withdraw, GemHeaderButtonAction.SendCollectible, GemHeaderButtonAction.CollectibleMenu -> Unit
                }
            }
        },
    )
}
