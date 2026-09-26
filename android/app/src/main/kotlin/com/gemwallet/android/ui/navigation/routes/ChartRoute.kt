package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.market.presents.ChartScreen
import com.gemwallet.android.ui.models.navigation.RouteMessage
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.ChainAddress
import kotlinx.serialization.Serializable

@Serializable
data class ChartRoute(val assetId: AssetId) : NavKey

fun EntryProviderScope<NavKey>.chartScreen(
    onPriceAlerts: (AssetId) -> Unit,
    onSetPriceAlert: (AssetId) -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
    routeMessage: (ChartRoute) -> RouteMessage?,
    onRouteMessageShown: (ChartRoute) -> Unit,
    onCancel: () -> Unit,
) {
    entry<ChartRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) { key ->
        ChartScreen(
            onPriceAlerts = onPriceAlerts,
            onSetPriceAlert = onSetPriceAlert,
            onOpenAddress = onOpenAddress,
            message = routeMessage(key),
            onMessageShown = { onRouteMessageShown(key) },
            onCancel = onCancel,
        )
    }
}
