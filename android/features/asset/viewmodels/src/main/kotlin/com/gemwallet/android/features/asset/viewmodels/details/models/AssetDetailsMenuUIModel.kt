package com.gemwallet.android.features.asset.viewmodels.details.models

import com.gemwallet.android.features.asset.viewmodels.style.symbol
import com.gemwallet.android.features.asset.viewmodels.localization.toastRes
import androidx.annotation.StringRes
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import uniffi.gemstone.GemAssetEmptyAction
import uniffi.gemstone.GemPriceAlertToggle

data class PriceAlertMenuUIModel(
    val needsPermission: Boolean,
    @param:StringRes val toastRes: Int,
    val symbol: ListItemSymbol,
)

data class EmptyTransactionsUIModel(
    val showsBuy: Boolean,
    val showsSwap: Boolean,
)

internal fun GemPriceAlertToggle.menu(): PriceAlertMenuUIModel = PriceAlertMenuUIModel(
    needsPermission = this == GemPriceAlertToggle.DISABLED,
    toastRes = toastRes(),
    symbol = symbol(),
)

internal fun GemAssetEmptyAction?.emptyTransactions(): EmptyTransactionsUIModel = EmptyTransactionsUIModel(
    showsBuy = this == GemAssetEmptyAction.BUY,
    showsSwap = this == GemAssetEmptyAction.SWAP,
)
