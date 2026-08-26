package com.gemwallet.android.features.nft.viewmodels

sealed interface NftListMode {

    val collectionId: String? get() = null

    data object Collections : NftListMode

    data class Collection(override val collectionId: String) : NftListMode

    data object Unverified : NftListMode
}
