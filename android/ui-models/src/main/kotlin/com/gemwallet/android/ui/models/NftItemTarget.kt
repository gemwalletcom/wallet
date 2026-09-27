package com.gemwallet.android.ui.models

import com.wallet.core.primitives.NFTAssetId
import uniffi.gemstone.GemNftEntry
import uniffi.gemstone.GemNftItem

sealed interface NftItemTarget {
    data class Collection(val id: String) : NftItemTarget
    data class Asset(val id: NFTAssetId) : NftItemTarget
}

val GemNftEntry.target: NftItemTarget
    get() = when (val item = item) {
        is GemNftItem.Collection -> NftItemTarget.Collection(item.data.collection.id)
        is GemNftItem.Asset -> NftItemTarget.Asset(NFTAssetId(item.data.asset.id))
    }
