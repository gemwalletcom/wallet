package com.wallet.core.primitives

import com.gemwallet.android.serializer.PerpetualIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = PerpetualIdSerializer::class)
data class PerpetualId(
    val provider: PerpetualProvider,
    val symbol: String,
)
