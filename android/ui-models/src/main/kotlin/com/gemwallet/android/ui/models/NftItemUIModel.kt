package com.gemwallet.android.ui.models

import com.wallet.core.primitives.NFTAssetId
import uniffi.gemstone.GemNftEntry
import uniffi.gemstone.GemNftItem
import uniffi.gemstone.GemNftRow

sealed interface NftItemTarget {
    data class Collection(val id: String) : NftItemTarget
    data class Asset(val id: NFTAssetId) : NftItemTarget
}

data class NftItemUIModel(val row: GemNftRow, val target: NftItemTarget) {
    val imageUrl: String get() = row.imageUrl
    val name: String get() = row.title
    val isVerified: Boolean get() = row.isVerified
    val countText: String? get() = row.countText
}

fun List<GemNftEntry>.toUIModels(): List<NftItemUIModel> = map { entry ->
    when (val item = entry.item) {
        is GemNftItem.Collection -> NftItemUIModel(entry.row, NftItemTarget.Collection(item.data.collection.id))
        is GemNftItem.Asset -> NftItemUIModel(entry.row, NftItemTarget.Asset(NFTAssetId(item.data.asset.id)))
    }
}
