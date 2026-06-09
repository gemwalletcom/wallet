package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.views.AssetsResultsScreen
import com.gemwallet.android.features.assets.views.WalletSearchAction
import com.gemwallet.android.features.assets.views.WalletSearchScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetTag
import kotlinx.serialization.Serializable

@Serializable
data object WalletSearchRoute : NavKey

@Serializable
data class AssetsResultsRoute(val query: String, val tag: AssetTag?) : NavKey

fun EntryProviderScope<NavKey>.walletSearchScreen(
    onAction: (WalletSearchAction) -> Unit,
) {
    entry<WalletSearchRoute> {
        WalletSearchScreen(onAction = onAction)
    }

    entry<AssetsResultsRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.Query to key.query,
                RouteArgument.Tag to key.tag?.string,
            )
        },
    ) {
        AssetsResultsScreen(onAction = onAction)
    }
}
