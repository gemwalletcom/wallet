package com.gemwallet.android.features.asset.viewmodels.details.models

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
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

internal fun GemPriceAlertToggle.menu(): PriceAlertMenuUIModel = when (this) {
    GemPriceAlertToggle.ENABLED -> PriceAlertMenuUIModel(needsPermission = false, toastRes = R.string.price_alerts_disabled_for, symbol = ListItemSymbol.Notifications)
    GemPriceAlertToggle.DISABLED -> PriceAlertMenuUIModel(needsPermission = true, toastRes = R.string.price_alerts_enabled_for, symbol = ListItemSymbol.NotificationsOutlined)
}

internal fun GemAssetEmptyAction?.emptyTransactions(): EmptyTransactionsUIModel = EmptyTransactionsUIModel(
    showsBuy = this == GemAssetEmptyAction.BUY,
    showsSwap = this == GemAssetEmptyAction.SWAP,
)
