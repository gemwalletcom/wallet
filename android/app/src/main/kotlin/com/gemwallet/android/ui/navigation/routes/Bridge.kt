package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.bridge.views.ConnectionScene
import com.gemwallet.android.features.bridge.views.ConnectionsScene
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import kotlinx.serialization.Serializable

@Serializable
data object BridgeConnectionsRoute : NavKey

@Serializable
data class BridgeConnectionDetailsRoute(val connectionId: String) : NavKey

fun EntryProviderScope<NavKey>.bridgesScreen(
    onConnection: (String) -> Unit,
    onCancel: () -> Unit,
) {
    entry<BridgeConnectionsRoute> {
        ConnectionsScene(
            onConnection = onConnection,
            onCancel = onCancel
        )
    }

    entry<BridgeConnectionDetailsRoute>(
        metadata = { key -> routeArguments(RouteArgument.ConnectionId to key.connectionId) },
    ) {
        ConnectionScene(onCancel = onCancel)
    }
}
