package com.gemwallet.android.features.assets.views.components

import androidx.compose.runtime.Composable
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.wallet.aggregates.WalletSummaryAggregate
import com.gemwallet.android.ui.components.HideToggle
import com.gemwallet.android.ui.components.list_head.AmountListHead
import com.gemwallet.android.ui.components.list_head.AssetHeadActions
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemHeaderButton
import uniffi.gemstone.GemHeaderButtonKind

@Composable
internal fun AssetsHead(
    walletSummary: WalletSummaryAggregate?,
    onSendClick: () -> Unit,
    onReceiveClick: () -> Unit,
    onBuyClick: () -> Unit,
    onSwapClick: () -> Unit,
    onHideBalances: () -> Unit,
    onPortfolio: () -> Unit,
) {
    walletSummary ?: return

    AmountListHead(
        amount = walletSummary.walletTotalValue,
        hideToggle = HideToggle(
            hidden = walletSummary.isBalanceHidden,
            onToggle = onHideBalances,
        ),
        changedValue = walletSummary.changedValue?.valueFormatted,
        changedPercentages = walletSummary.changedValue?.changePercentageFormatted,
        changeState = walletSummary.changedValue?.state ?: ValueDirection.None,
        onSubtitleClick = onPortfolio,
        actions = {
            AssetHeadActions(
                isViewOnly = walletSummary.walletType == WalletType.View,
                buttons = listOfNotNull(
                    GemHeaderButtonKind.SEND,
                    GemHeaderButtonKind.RECEIVE,
                    GemHeaderButtonKind.BUY,
                    GemHeaderButtonKind.SWAP.takeIf { walletSummary.isSwapAvailable },
                ).map { GemHeaderButton(it, isEnabled = walletSummary.isOperationsAvailable) },
                onTransfer = onSendClick,
                onReceive = onReceiveClick,
                onBuy = onBuyClick,
                onSwap = onSwapClick,
            )
        }
    )
}
