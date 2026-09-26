package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.wallet_connector.presents.ConnectionScreen
import com.gemwallet.android.features.wallet_connector.presents.ConnectionsScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data object ConnectionsRoute : NavKey

@Serializable
data class ConnectionRoute(val connectionId: String) : NavKey

fun EntryProviderScope<NavKey>.connectionsScreen(onConnection: (String) -> Unit, onCancel: () -> Unit) {
    entry<ConnectionsRoute> {
        ConnectionsScreen(
            onConnection = onConnection,
            onCancel = onCancel,
        )
    }

    entry<ConnectionRoute>(
        metadata = { key -> routeArguments(RouteArgument.ConnectionId to key.connectionId) },
    ) {
        ConnectionScreen(onCancel = onCancel)
    }
}
