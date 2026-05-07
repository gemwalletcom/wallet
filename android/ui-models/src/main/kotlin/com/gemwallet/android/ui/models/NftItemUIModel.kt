package com.gemwallet.android.ui.models

import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.VerificationStatus

data class NftItemUIModel(
    val collection: NFTCollection,
    val asset: NFTAsset? = null,
    val collectionSize: Int? = null,
) {
    val imageUrl: String get() = asset?.images?.preview?.url ?: collection.images.preview.url
    val name: String get() = asset?.name ?: collection.name
    val isVerified: Boolean get() = collection.status == VerificationStatus.Verified
}
