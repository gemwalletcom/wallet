package com.wallet.core.primitives

import com.gemwallet.android.serializer.PerpetualIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = PerpetualIdSerializer::class)
data class PerpetualId(
    val provider: PerpetualProvider,
    val symbol: String,
) {
    constructor(identifier: String) : this(
        provider = PerpetualProvider.entries.firstOrNull { it.string == identifier.substringBefore("_") }
            ?: throw IllegalArgumentException("Invalid perpetual id: $identifier"),
        symbol = identifier.substringAfter("_", "").ifEmpty { throw IllegalArgumentException("Invalid perpetual id: $identifier") },
    )
}
