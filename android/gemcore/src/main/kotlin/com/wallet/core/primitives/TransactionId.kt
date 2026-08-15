package com.wallet.core.primitives

import com.gemwallet.android.serializer.TransactionIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = TransactionIdSerializer::class)
data class TransactionId(
    val chain: Chain,
    val hash: String,
) {
    companion object {
        fun from(id: String): TransactionId? {
            val components = id.split("_", limit = 2)
            if (components.size != 2) {
                return null
            }

            val chain = Chain.entries.firstOrNull { it.string == components[0] } ?: return null
            return TransactionId(chain, components[1])
        }
    }

    val identifier: String
        get() = "${chain.string}_$hash"

    override fun toString(): String = identifier
}
