package com.wallet.core.primitives

import com.gemwallet.android.serializer.NFTCollectionIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = NFTCollectionIdSerializer::class)
data class NFTCollectionId(
    val chain: Chain,
    val contractAddress: String,
)
