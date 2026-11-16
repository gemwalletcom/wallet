package com.gemwallet.android.ui.navigation.routes

import androidx.compose.runtime.Composable
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.nft.presents.CollectibleScreen
import com.gemwallet.android.features.nft.presents.CollectionsScreen
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.NftAssetIdAction
import com.gemwallet.android.ui.models.actions.NftCollectionIdAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.NFTAsset
import kotlinx.serialization.Serializable

@Serializable
data object CollectionsRoute : NavKey

@Serializable
data class CollectionRoute(val nftCollectionId: String) : NavKey

@Serializable
data object UnverifiedCollectionsRoute : NavKey

@Serializable
data class CollectibleRoute(val nftAssetId: String) : NavKey

fun EntryProviderScope<NavKey>.collectionsScreen(
    cancelAction: CancelAction,
    onRecipient: (NFTAsset) -> Unit,
    onReceive: () -> Unit,
    onUnverified: () -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
    collectionIdAction: NftCollectionIdAction,
    assetIdAction: NftAssetIdAction,
) {
    val collections: @Composable () -> Unit = {
        CollectionsScreen(
            cancelAction = cancelAction,
            collectionAction = collectionIdAction,
            assetAction = assetIdAction,
            onReceive = onReceive,
            onUnverified = onUnverified,
        )
    }

    entry<CollectionsRoute> {
        collections()
    }

    entry<CollectionRoute>(
        metadata = { key -> routeArguments(RouteArgument.NftCollectionId to key.nftCollectionId) },
    ) {
        collections()
    }

    entry<UnverifiedCollectionsRoute>(
        metadata = { routeArguments(RouteArgument.Unverified to true) },
    ) {
        collections()
    }

    entry<CollectibleRoute>(
        metadata = { key -> routeArguments(RouteArgument.NftAssetId to key.nftAssetId) },
    ) {
        CollectibleScreen(cancelAction, onRecipient, onOpenAddress)
    }
}
