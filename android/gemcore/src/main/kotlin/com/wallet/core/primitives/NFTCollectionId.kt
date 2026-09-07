package com.wallet.core.primitives

import com.gemwallet.android.serializer.NFTCollectionIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = NFTCollectionIdSerializer::class)
data class NFTCollectionId(
    val chain: Chain,
    val contractAddress: String,
) {
    constructor(identifier: String) : this(
        chain = Chain.entries.firstOrNull { it.string == identifier.substringBefore("_") }
            ?: throw IllegalArgumentException("Invalid nft collection id: $identifier"),
        contractAddress = identifier.substringAfter("_", "").ifEmpty { throw IllegalArgumentException("Invalid nft collection id: $identifier") },
    )
}
