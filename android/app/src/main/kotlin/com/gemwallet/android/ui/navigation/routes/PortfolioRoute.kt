package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.wallet_tab.presents.PortfolioScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.PortfolioType
import kotlinx.serialization.Serializable

@Serializable
data class PortfolioRoute(val type: PortfolioType = PortfolioType.Wallet) : NavKey

fun EntryProviderScope<NavKey>.portfolioScreen(onCancel: () -> Unit) {
    entry<PortfolioRoute>(
        metadata = { key -> routeArguments(RouteArgument.Type to key.type) },
    ) {
        PortfolioScreen(onCancel = onCancel)
    }
}
