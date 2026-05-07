package com.gemwallet.android.ui.navigation.routes

import androidx.compose.ui.res.stringResource
import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.nft.presents.NFTDetailsScene
import com.gemwallet.android.features.nft.presents.NftListNavScreen
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.NftAssetIdAction
import com.gemwallet.android.ui.models.actions.NftCollectionIdAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.AssetId
import kotlinx.serialization.Serializable

const val nftRoute = "nft"

@Serializable
data class NftCollectionRoute(val nftCollectionId: String) : NavKey

@Serializable
data object NftUnverifiedCollectionsRoute : NavKey

@Serializable
data class NftAssetRoute(val nftAssetId: String) : NavKey

fun EntryProviderScope<NavKey>.nftCollection(
    cancelAction: CancelAction,
    onRecipient: (AssetId, String) -> Unit,
    onReceive: () -> Unit,
    collectionIdAction: NftCollectionIdAction,
    assetIdAction: NftAssetIdAction,
) {
    entry<NftCollectionRoute>(
        metadata = { key -> routeArguments(RouteArgument.NftCollectionId to key.nftCollectionId) },
    ) {
        NftListNavScreen(cancelAction, collectionIdAction, assetIdAction, onReceive = onReceive)
    }

    entry<NftUnverifiedCollectionsRoute>(
        metadata = { routeArguments(RouteArgument.Unverified to true) },
    ) {
        NftListNavScreen(
            cancelAction = cancelAction,
            collectionAction = collectionIdAction,
            assetAction = assetIdAction,
            title = stringResource(R.string.asset_verification_unverified),
        )
    }

    entry<NftAssetRoute>(
        metadata = { key -> routeArguments(RouteArgument.NftAssetId to key.nftAssetId) },
    ) {
        NFTDetailsScene(cancelAction, onRecipient)
    }
}
