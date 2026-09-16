package com.gemwallet.android.ui.models

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTCollection
import uniffi.gemstone.GemNftItem
import uniffi.gemstone.GemNftRow
import uniffi.gemstone.nftRows

data class NftItemUIModel(
    val collection: NFTCollection,
    val asset: NFTAsset? = null,
    val row: GemNftRow,
) {
    val imageUrl: String get() = row.imageUrl
    val name: String get() = row.title
    val isVerified: Boolean get() = row.isVerified
    val collectionSize: Int? get() = row.count?.toInt()
}

fun List<GemNftItem>.toUIModels(): List<NftItemUIModel> = zip(nftRows(this)) { item, row ->
    when (item) {
        is GemNftItem.Collection -> item.data.toPrimitives().let { NftItemUIModel(it.collection, null, row) }
        is GemNftItem.Asset -> item.data.toPrimitives().let { NftItemUIModel(it.collection, it.asset, row) }
    }
}
