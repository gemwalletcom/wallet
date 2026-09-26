package com.gemwallet.android.ui.components.image

import com.gemwallet.android.ui.models.NftItemUIModel
import com.wallet.core.primitives.NFTAsset

data class NftImageSource(val url: String, val name: String)

fun NFTAsset.toImageSource(): NftImageSource = NftImageSource(url = images.preview.url, name = name)

fun NftItemUIModel.toImageSource(): NftImageSource = NftImageSource(
    url = imageUrl,
    name = name,
)
