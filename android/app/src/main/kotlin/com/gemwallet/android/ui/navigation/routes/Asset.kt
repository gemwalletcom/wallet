package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.features.asset.presents.chart.AssetChartScene
import com.gemwallet.android.features.asset.presents.details.AssetDetailsAction
import com.gemwallet.android.features.asset.presents.details.AssetDetailsScreen
import com.gemwallet.android.features.assets.views.HiddenAssetsAction
import com.gemwallet.android.features.assets.views.HiddenAssetsScreen
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

const val assetsRoute = "assets"

@Serializable
data class AssetRoute(val assetId: AssetId) : NavKey

@Serializable
data class AssetChartRoute(val assetId: AssetId) : NavKey

@Serializable
data object HiddenAssetsRoute : NavKey

fun EntryProviderScope<NavKey>.assetScreen(
    onAction: (AssetDetailsAction.Navigation) -> Unit,
) {
    entry<AssetRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        AssetDetailsScreen(onAction = onAction)
    }
}

fun EntryProviderScope<NavKey>.hiddenAssetsScreen(
    onOpenAsset: (AssetId) -> Unit,
    onCancel: () -> Unit,
) {
    entry<HiddenAssetsRoute> {
        HiddenAssetsScreen(
            onAction = { action ->
                when (action) {
                    HiddenAssetsAction.Close -> onCancel()
                    is HiddenAssetsAction.OpenAsset -> onOpenAsset(action.assetId)
                }
            },
        )
    }
}

fun EntryProviderScope<NavKey>.assetChartScreen(
    onPriceAlerts: (AssetId) -> Unit,
    onAddPriceAlertTarget: (AssetId) -> Unit,
    toastMessage: (AssetChartRoute) -> String?,
    onToastShown: (AssetChartRoute) -> Unit,
    onCancel: () -> Unit,
) {
    entry<AssetChartRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) { key ->
        AssetChartScene(
            onPriceAlerts = onPriceAlerts,
            onAddPriceAlertTarget = onAddPriceAlertTarget,
            toastMessage = toastMessage(key),
            onToastShown = { onToastShown(key) },
            onCancel = onCancel,
        )
    }
}
