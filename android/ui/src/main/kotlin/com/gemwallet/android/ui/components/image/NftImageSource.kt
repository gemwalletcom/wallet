package com.gemwallet.android.ui.components.image

import com.wallet.core.primitives.NFTAsset
import uniffi.gemstone.GemNftRow

data class NftImageSource(val url: String, val name: String)

fun NFTAsset.toImageSource(): NftImageSource = NftImageSource(url = images.preview.url, name = name)

fun GemNftRow.toImageSource(): NftImageSource = NftImageSource(
    url = imageUrl,
    name = title,
)
