package com.wallet.core.primitives

import com.gemwallet.android.serializer.TransactionIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = TransactionIdSerializer::class)
data class TransactionId(
    val chain: Chain,
    val hash: String,
) {
    constructor(identifier: String) : this(
        chain = Chain.entries.firstOrNull { it.string == identifier.substringBefore("_") }
            ?: throw IllegalArgumentException("Invalid transaction id: $identifier"),
        hash = identifier.substringAfter("_", "").ifEmpty { throw IllegalArgumentException("Invalid transaction id: $identifier") },
    )

    companion object {
        fun from(id: String): TransactionId? = runCatching { TransactionId(id) }.getOrNull()
    }

    val identifier: String
        get() = "${chain.string}_$hash"

    override fun toString(): String = identifier
}
