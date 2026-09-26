package com.gemwallet.android.features.assets.viewmodels.details.models

import androidx.annotation.StringRes
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.localization.toastRes
import com.gemwallet.android.ui.style.symbol
import uniffi.gemstone.GemAssetEmptyAction
import uniffi.gemstone.GemPriceAlertToggle

data class PriceAlertMenuUIModel(@param:StringRes val toastRes: Int, val symbol: ListItemSymbol)

data class EmptyTransactionsUIModel(val showsBuy: Boolean, val showsSwap: Boolean)

internal fun GemPriceAlertToggle.menu(): PriceAlertMenuUIModel = PriceAlertMenuUIModel(
    toastRes = toastRes(),
    symbol = symbol(),
)

internal fun GemAssetEmptyAction?.emptyTransactions(): EmptyTransactionsUIModel = EmptyTransactionsUIModel(
    showsBuy = this == GemAssetEmptyAction.BUY,
    showsSwap = this == GemAssetEmptyAction.SWAP,
)
