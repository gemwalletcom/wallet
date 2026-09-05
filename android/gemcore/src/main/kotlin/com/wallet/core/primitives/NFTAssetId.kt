package com.wallet.core.primitives

import com.gemwallet.android.serializer.NFTAssetIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = NFTAssetIdSerializer::class)
data class NFTAssetId(
    val chain: Chain,
    val contractAddress: String,
    val tokenId: String,
) {
    constructor(identifier: String) : this(
        chain = Chain.entries.firstOrNull { it.string == identifier.substringBefore("_") }
            ?: throw IllegalArgumentException("Invalid nft asset id: $identifier"),
        contractAddress = identifier.substringAfter("_", "").substringBefore("::", "").ifEmpty { throw IllegalArgumentException("Invalid nft asset id: $identifier") },
        tokenId = identifier.substringAfter("::", "").ifEmpty { throw IllegalArgumentException("Invalid nft asset id: $identifier") },
    )
}
