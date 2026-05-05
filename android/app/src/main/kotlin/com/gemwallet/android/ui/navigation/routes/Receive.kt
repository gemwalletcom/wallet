package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset_select.presents.views.SelectReceiveScreen
import com.gemwallet.android.features.receive.presents.ReceiveNftChainsScreen
import com.gemwallet.android.features.receive.presents.ReceiveScreen
import com.gemwallet.android.ui.navigation.assetIdArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

@Serializable
data class ReceiveRoute(val assetId: AssetId) : NavKey

@Serializable
data object ReceiveSelectRoute : NavKey

@Serializable
data object ReceiveNftChainsRoute : NavKey

fun EntryProviderScope<NavKey>.receiveScreen(
    onCancel: () -> Unit,
    onReceive: (AssetId) -> Unit,
) {
    entry<ReceiveRoute>(
        metadata = { key -> routeArguments(assetIdArgument(key.assetId)) },
    ) {
        ReceiveScreen(onCancel = onCancel)
    }

    entry<ReceiveSelectRoute> {
        SelectReceiveScreen(
            onCancel = onCancel,
            onSelect = onReceive,
        )
    }

    entry<ReceiveNftChainsRoute> {
        ReceiveNftChainsScreen(
            onCancel = onCancel,
            onSelect = { onReceive(AssetId(it)) },
        )
    }
}
