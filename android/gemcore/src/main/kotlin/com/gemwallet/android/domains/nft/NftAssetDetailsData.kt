package com.gemwallet.android.domains.nft

import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTCollection
import uniffi.gemstone.GemCollectibleDetails

data class NftAssetDetailsData(
    val collection: NFTCollection,
    val asset: NFTAsset,
    val details: GemCollectibleDetails,
)
