package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset.presents.chart.PortfolioChartScene
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.PortfolioType
import kotlinx.serialization.Serializable

@Serializable
data class PortfolioChartRoute(val type: PortfolioType = PortfolioType.Wallet) : NavKey

fun EntryProviderScope<NavKey>.portfolioChartScreen(
    onCancel: () -> Unit,
) {
    entry<PortfolioChartRoute>(
        metadata = { key -> routeArguments(RouteArgument.Type to key.type) },
    ) {
        PortfolioChartScene(onCancel = onCancel)
    }
}
