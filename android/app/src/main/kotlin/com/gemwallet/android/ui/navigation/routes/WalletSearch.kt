package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.views.AssetsResultsScreen
import com.gemwallet.android.features.assets.views.WalletSearchAction
import com.gemwallet.android.features.assets.views.WalletSearchScreen
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.domains.search.encode
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data object WalletSearchRoute : NavKey

@Serializable
data class AssetsResultsRoute(
    val query: String,
    val scope: WalletSearchTag,
    val title: String? = null,
) : NavKey

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
                RouteArgument.Scope to key.scope.encode(),
                RouteArgument.Title to key.title,
            )
        },
    ) {
        AssetsResultsScreen(onAction = onAction)
    }
}
