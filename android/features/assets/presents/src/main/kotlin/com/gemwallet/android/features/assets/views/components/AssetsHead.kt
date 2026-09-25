package com.gemwallet.android.features.assets.views.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import com.gemwallet.android.domains.wallet.aggregates.WalletSummary
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.HideToggle
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.gemwallet.android.ui.components.list_head.uiModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.style.textStyle

@Composable
internal fun AssetsHead(walletSummary: WalletSummary?, onSendClick: () -> Unit, onReceiveClick: () -> Unit, onBuyClick: () -> Unit, onSwapClick: () -> Unit, onHideBalances: () -> Unit, onPortfolio: () -> Unit) {
    walletSummary ?: return

    AmountListHead(
        amount = walletSummary.state.total.text(),
        hideToggle = HideToggle(
            hidden = walletSummary.isBalanceHidden,
            onToggle = onHideBalances,
        ),
        changedValue = walletSummary.state.pnl?.string(LocalContext.current),
        changeStyle = walletSummary.state.pnlTone.textStyle(),
        onSubtitleClick = onPortfolio,
        actions = {
            AssetHeadActions(
                walletSummary.state.headerActions.uiModel(
                    onTransfer = onSendClick,
                    onReceive = onReceiveClick,
                    onBuy = onBuyClick,
                    onSwap = onSwapClick,
                ),
            )
        },
    )
}
