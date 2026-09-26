package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.presents.select.SelectReceiveCollectionScreen
import com.gemwallet.android.features.assets.presents.select.SelectReceiveScreen
import com.gemwallet.android.features.transfer.presents.receive.ReceiveScreen
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

@Serializable
data class ReceiveRoute(val assetId: AssetId) : NavKey

@Serializable
data object ReceiveSelectRoute : NavKey

@Serializable
data object ReceiveCollectionRoute : NavKey

fun EntryProviderScope<NavKey>.receiveScreen(onCancel: () -> Unit, onReceive: (AssetId) -> Unit) {
    entry<ReceiveRoute> { key ->
        ReceiveScreen(assetId = key.assetId, onCancel = onCancel)
    }

    entry<ReceiveSelectRoute> {
        SelectReceiveScreen(
            onCancel = onCancel,
            onSelect = onReceive,
        )
    }

    entry<ReceiveCollectionRoute> {
        SelectReceiveCollectionScreen(
            onCancel = onCancel,
            onSelect = onReceive,
        )
    }
}
