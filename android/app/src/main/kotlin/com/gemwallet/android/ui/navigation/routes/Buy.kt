package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.presents.select.SelectBuyScreen
import com.gemwallet.android.features.fiat_connect.presents.FiatScreen
import com.gemwallet.android.features.fiat_connect.presents.FiatTransactionsScreen
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.fiatAmountArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.FiatQuoteType
import kotlinx.serialization.Serializable

@Serializable
data class FiatInputRoute(val assetId: AssetId, val amount: Int? = null, val type: FiatQuoteType = FiatQuoteType.Buy) : NavKey

@Serializable
data object FiatSelectRoute : NavKey

@Serializable
data object FiatTransactionsRoute : NavKey

fun EntryProviderScope<NavKey>.fiatScreen(cancelAction: CancelAction, onBuy: (AssetId) -> Unit, onFiatTransactions: () -> Unit) {
    entry<FiatInputRoute>(
        metadata = { key ->
            routeArguments(
                assetIdArgument(key.assetId),
                fiatAmountArgument(key.amount),
                RouteArgument.Type to key.type,
            )
        },
    ) {
        FiatScreen(
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
        FiatTransactionsScreen(
            onClose = cancelAction,
        )
    }
}
