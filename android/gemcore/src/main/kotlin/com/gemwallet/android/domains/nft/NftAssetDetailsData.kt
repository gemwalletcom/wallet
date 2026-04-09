package com.gemwallet.android.domains.nft

import com.wallet.core.primitives.Account
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAttribute
import com.wallet.core.primitives.NFTCollection

data class NftAssetDetailsData(
    val collection: NFTCollection,
    val asset: NFTAsset,
    val account: Account,
    val contractExplorerLink: BlockExplorerLink? = null,
    val tokenIdExplorerLink: BlockExplorerLink? = null,
) {
    val imageUrl: String get() = asset.images.preview.url
    val assetName: String get() = asset.name
    val collectionName: String get() = collection.name
    val description: String? get() = asset.description
    val attributes: List<NFTAttribute> get() = asset.attributes
}
