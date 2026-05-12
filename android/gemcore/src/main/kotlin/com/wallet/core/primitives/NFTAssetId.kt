package com.wallet.core.primitives

import com.gemwallet.android.serializer.NFTAssetIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = NFTAssetIdSerializer::class)
data class NFTAssetId(
    val chain: Chain,
    val contractAddress: String,
    val tokenId: String,
)
