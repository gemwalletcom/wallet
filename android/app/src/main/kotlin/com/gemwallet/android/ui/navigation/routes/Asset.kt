package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.features.asset.presents.chart.AssetChartScene
import com.gemwallet.android.features.asset.presents.details.AssetDetailsScreen
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.Serializable

const val assetsRoute = "assets"

@Serializable
data class AssetRoute(val assetId: AssetId) : NavKey

@Serializable
data class AssetChartRoute(val assetId: AssetId) : NavKey

fun EntryProviderScope<NavKey>.assetScreen(
    onCancel: () -> Unit,
    onTransfer: AssetIdAction,
    onReceive: (AssetId) -> Unit,
    onBuy: (AssetId) -> Unit,
    onSwap: (AssetId, AssetId?) -> Unit,
    onTransaction: (TransactionId) -> Unit,
    onChart: (AssetId) -> Unit,
    openNetwork: AssetIdAction,
    onStake: (AssetId) -> Unit,
    onConfirm: (ConfirmParams) -> Unit,
    onPriceAlerts: (AssetId) -> Unit,
) {
    entry<AssetRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        AssetDetailsScreen(
            onCancel = onCancel,
            onTransfer = onTransfer,
            onReceive = onReceive,
            onBuy = onBuy,
            onSwap = onSwap,
            onTransaction = onTransaction,
            onChart = onChart,
            openNetwork = openNetwork,
            onStake = onStake,
            onConfirm = onConfirm,
            onPriceAlerts = onPriceAlerts
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
