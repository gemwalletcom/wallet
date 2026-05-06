package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.features.swap.views.SwapScreen
import com.gemwallet.android.features.swap.views.SwapSelectScreen
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.WalletNavigator
import com.gemwallet.android.ui.navigation.fromAssetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.gemwallet.android.ui.navigation.toAssetIdArgument
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

@Serializable
data object SwapRoute : NavKey

@Serializable
data class SwapPairRoute(val from: AssetId, val to: AssetId?) : NavKey

@Serializable
data class SwapSelectRoute(
    val itemType: SwapItemType,
    val payAssetId: AssetId?,
    val receiveAssetId: AssetId?,
) : NavKey

fun EntryProviderScope<NavKey>.swapSelect(navigator: WalletNavigator, onCancel: () -> Unit) {
    entry<SwapSelectRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.SwapItemType to key.itemType,
                fromAssetIdArgument(key.payAssetId),
                toAssetIdArgument(key.receiveAssetId),
            )
        },
    ) {
        SwapSelectScreen(
            onCancel,
            navigator::finishSwapSelect,
        )
    }
}

fun EntryProviderScope<NavKey>.swap(
    navigator: WalletNavigator,
    onConfirm: (ConfirmParams) -> Unit,
    onSelect: (select: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) -> Unit,
    onCancel: () -> Unit,
) {
    entry<SwapRoute> {
        swapScreenContent(navigator, SwapRoute, from = null, to = null, onConfirm, onSelect, onCancel)
    }

    entry<SwapPairRoute>(
        metadata = { key ->
            routeArguments(
                fromAssetIdArgument(key.from),
                toAssetIdArgument(key.to),
            )
        },
    ) { key ->
        swapScreenContent(navigator, key, key.from, key.to, onConfirm, onSelect, onCancel)
    }
}

@Composable
private fun swapScreenContent(
    navigator: WalletNavigator,
    route: NavKey,
    from: AssetId?,
    to: AssetId?,
    onConfirm: (ConfirmParams) -> Unit,
    onSelect: (select: SwapItemType, payAssetId: AssetId?, receiveAssetId: AssetId?) -> Unit,
    onCancel: () -> Unit,
) {
    val selection = navigator.swapSelection(route)
    SwapScreen(
        payId = selection?.payAssetId ?: from,
        receiveId = selection?.receiveAssetId ?: to,
        select = selection?.itemType,
        onSelectionConsumed = { navigator.clearSwapSelection(route) },
        onConfirm = onConfirm,
        onSelect = onSelect,
        onCancel = onCancel,
    )
}
