package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset.presents.chart.AssetChartScene
import com.gemwallet.android.features.asset.presents.details.AssetDetailsScreen
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetDetailsAction
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import kotlinx.serialization.Serializable

const val assetsRoute = "assets"

@Serializable
data class AssetRoute(val assetId: AssetId) : NavKey

@Serializable
data class AssetChartRoute(val assetId: AssetId) : NavKey

fun EntryProviderScope<NavKey>.assetScreen(onAction: (AssetDetailsAction.Navigation) -> Unit) {
    entry<AssetRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        AssetDetailsScreen(onAction = onAction)
    }
}

fun EntryProviderScope<NavKey>.assetChartScreen(
    onPriceAlerts: (AssetId) -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
    routeMessage: (AssetChartRoute) -> RouteMessage?,
    onRouteMessageShown: (AssetChartRoute) -> Unit,
    onCancel: () -> Unit,
) {
    entry<AssetChartRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) { key ->
        AssetChartScene(
            onPriceAlerts = onPriceAlerts,
            onAddPriceAlertTarget = onAddPriceAlertTarget,
            onOpenAddress = onOpenAddress,
            message = routeMessage(key),
            onMessageShown = { onRouteMessageShown(key) },
            onCancel = onCancel,
        )
    }
}
