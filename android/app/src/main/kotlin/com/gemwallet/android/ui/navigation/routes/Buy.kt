package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset_select.presents.views.SelectBuyScreen
import com.gemwallet.android.features.buy.views.FiatNavScreen
import com.gemwallet.android.features.buy.views.FiatTransactionsNavScreen
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.fiatAmountArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

@Serializable
data class FiatInputRoute(val assetId: AssetId, val amount: Double? = null) : NavKey

@Serializable
data object FiatSelectRoute : NavKey

@Serializable
data object FiatTransactionsRoute : NavKey

fun EntryProviderScope<NavKey>.fiatScreen(
    cancelAction: CancelAction,
    onBuy: (AssetId) -> Unit,
    onFiatTransactions: () -> Unit,
) {
    entry<FiatInputRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId), fiatAmountArgument(key.amount)) },
    ) {
        FiatNavScreen(
            cancelAction = cancelAction,
            onFiatTransactions = onFiatTransactions,
        )
    }

    entry<FiatSelectRoute> {
        SelectBuyScreen(
            cancelAction = cancelAction,
            onSelect = onBuy,
        )
    }

    entry<FiatTransactionsRoute> {
        FiatTransactionsNavScreen(
            onClose = cancelAction,
        )
    }
}
