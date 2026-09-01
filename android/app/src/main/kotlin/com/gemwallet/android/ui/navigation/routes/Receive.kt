package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.asset_select.presents.views.SelectReceiveScreen
import com.gemwallet.android.features.receive.presents.ReceiveNftChainsScreen
import com.gemwallet.android.features.receive.presents.ReceiveScreen
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
    entry<ReceiveRoute> { key ->
        ReceiveScreen(assetId = key.assetId, onCancel = onCancel)
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
