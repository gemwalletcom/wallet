package com.gemwallet.android.features.nft.presents

internal sealed interface NftListAction {
    data object Refresh : NftListAction
    data object Close : NftListAction
    data object Receive : NftListAction
    data object OpenUnverified : NftListAction
    data class OpenCollection(val collectionId: String) : NftListAction
    data class OpenAsset(val assetId: String) : NftListAction
}
