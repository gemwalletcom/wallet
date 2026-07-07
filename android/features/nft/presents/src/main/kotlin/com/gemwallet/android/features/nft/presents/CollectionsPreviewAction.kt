package com.gemwallet.android.features.nft.presents

import com.wallet.core.primitives.NFTAssetId

sealed interface CollectionsPreviewAction {
    data object OpenCollections : CollectionsPreviewAction
    data class OpenCollection(val collectionId: String) : CollectionsPreviewAction
    data class OpenAsset(val assetId: NFTAssetId) : CollectionsPreviewAction
}
