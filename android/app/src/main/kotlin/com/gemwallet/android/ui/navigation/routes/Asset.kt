package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.presents.asset.AssetScreen
import com.gemwallet.android.features.assets.viewmodels.asset.models.AssetAction
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

const val WalletRoute = "assets"

@Serializable
data class AssetRoute(val assetId: AssetId) : NavKey

fun EntryProviderScope<NavKey>.assetScreen(onAction: (AssetAction.Navigation) -> Unit) {
    entry<AssetRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        AssetScreen(onAction = onAction)
    }
}
