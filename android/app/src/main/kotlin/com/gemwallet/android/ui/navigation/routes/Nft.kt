package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.nft.presents.NFTDetailsScene
import com.gemwallet.android.features.nft.presents.NftListScreen
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.NftAssetIdAction
import com.gemwallet.android.ui.models.actions.NftCollectionIdAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.NFTAsset
import kotlinx.serialization.Serializable

const val nftRoute = "nft"

@Serializable
data object NftListRoute : NavKey

@Serializable
data class NftCollectionRoute(val nftCollectionId: String) : NavKey

@Serializable
data object NftUnverifiedCollectionsRoute : NavKey

@Serializable
data class NftAssetRoute(val nftAssetId: String) : NavKey

fun EntryProviderScope<NavKey>.nftCollection(
    cancelAction: CancelAction,
    onRecipient: (NFTAsset) -> Unit,
    onReceive: () -> Unit,
    onUnverified: () -> Unit,
    onOpenAddress: (ChainAddress) -> Unit,
    collectionIdAction: NftCollectionIdAction,
    assetIdAction: NftAssetIdAction,
) {
    entry<NftListRoute> {
        NftListScreen(
            cancelAction = cancelAction,
            collectionAction = collectionIdAction,
            assetAction = assetIdAction,
            onReceive = onReceive,
            onUnverified = onUnverified,
        )
    }

    entry<NftCollectionRoute>(
        metadata = { key -> routeArguments(RouteArgument.NftCollectionId to key.nftCollectionId) },
    ) {
        NftListScreen(
            cancelAction = cancelAction,
            collectionAction = collectionIdAction,
            assetAction = assetIdAction,
            onReceive = onReceive,
            onUnverified = onUnverified,
        )
    }

    entry<NftUnverifiedCollectionsRoute>(
        metadata = { routeArguments(RouteArgument.Unverified to true) },
    ) {
        NftListScreen(
            cancelAction = cancelAction,
            collectionAction = collectionIdAction,
            assetAction = assetIdAction,
            onReceive = onReceive,
            onUnverified = onUnverified,
        )
    }

    entry<NftAssetRoute>(
        metadata = { key -> routeArguments(RouteArgument.NftAssetId to key.nftAssetId) },
    ) {
        NFTDetailsScene(cancelAction, onRecipient, onOpenAddress)
    }
}
