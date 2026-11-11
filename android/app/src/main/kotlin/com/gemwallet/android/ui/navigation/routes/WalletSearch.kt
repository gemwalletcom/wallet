package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.wallet.presents.AssetsResultsScreen
import com.gemwallet.android.features.wallet.presents.WalletSearchAction
import com.gemwallet.android.features.wallet.presents.WalletSearchScreen
import com.gemwallet.android.serializer.GemSearchScopeSerializer
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemSearchScope

@Serializable
data object WalletSearchRoute : NavKey

@Serializable
data class AssetsResultsRoute(
    val query: String,
    val scope:
    @Serializable(with = GemSearchScopeSerializer::class)
    GemSearchScope,
    val title: String? = null,
) : NavKey

fun EntryProviderScope<NavKey>.walletSearchScreen(onAction: (WalletSearchAction) -> Unit) {
    entry<WalletSearchRoute> {
        WalletSearchScreen(onAction = onAction)
    }

    entry<AssetsResultsRoute>(
        metadata = { key ->
            routeArguments(
                RouteArgument.Query to key.query,
                RouteArgument.Scope to key.scope.packRoutePayload(),
                RouteArgument.Title to key.title,
            )
        },
    ) {
        AssetsResultsScreen(onAction = onAction)
    }
}
