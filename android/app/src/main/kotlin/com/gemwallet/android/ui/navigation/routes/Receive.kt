package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.assets.presents.select.SelectAssetScreen
import com.gemwallet.android.features.transfer.presents.receive.ReceiveScreen
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable
import uniffi.gemstone.GemSelectAssetType

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
        SelectAssetScreen(
            selectType = GemSelectAssetType.Receive,
            onCancel = onCancel,
            onSelect = onReceive,
        )
    }

    entry<ReceiveCollectionRoute> {
        SelectAssetScreen(
            selectType = GemSelectAssetType.ReceiveCollection,
            onCancel = onCancel,
            onSelect = onReceive,
        )
    }
}
