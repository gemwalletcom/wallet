package com.gemwallet.android.ui.components.image

import com.gemwallet.android.domains.asset.getImageUrl
import com.gemwallet.android.ui.models.NftItemUIModel
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.TransactionNFTTransferMetadata

data class NftImageSource(
    val url: String,
    val name: String,
)

fun NFTAsset.toImageSource(): NftImageSource =
    NftImageSource(url = getImageUrl(), name = name)

fun TransactionNFTTransferMetadata.toImageSource(): NftImageSource =
    NftImageSource(url = getImageUrl(), name = name.orEmpty())

fun NftItemUIModel.toImageSource(): NftImageSource =
    NftImageSource(
        url = asset?.getImageUrl() ?: collection.images.preview.url,
        name = name,
    )
