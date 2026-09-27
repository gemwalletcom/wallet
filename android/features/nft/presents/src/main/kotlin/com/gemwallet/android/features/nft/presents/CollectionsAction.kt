package com.gemwallet.android.features.nft.presents

import com.wallet.core.primitives.NFTAssetId

internal sealed interface CollectionsAction {
    data object Refresh : CollectionsAction
    data object Close : CollectionsAction
    data object Receive : CollectionsAction
    data object OpenUnverified : CollectionsAction
    data class OpenCollection(val collectionId: String) : CollectionsAction
    data class OpenAsset(val assetId: NFTAssetId) : CollectionsAction
}
