package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.views.NetworkAssetsScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import kotlinx.serialization.Serializable

@Serializable
data class NetworkAssetsRoute(val chain: Chain) : NavKey

fun EntryProviderScope<NavKey>.networkAssetsScreen(
    onSelectAsset: (AssetId) -> Unit,
    onManageAssets: (Chain) -> Unit,
    onCancel: () -> Unit,
) {
    entry<NetworkAssetsRoute>(
        metadata = { key -> routeArguments(RouteArgument.Chain to key.chain.string) },
    ) { key ->
        NetworkAssetsScreen(
            onSelectAsset = onSelectAsset,
            onManageAssets = { onManageAssets(key.chain) },
            onCancel = onCancel,
        )
    }
}
