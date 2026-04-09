package com.wallet.core.primitives

import com.gemwallet.android.serializer.WalletIdSerializer
import kotlinx.serialization.Serializable

@Serializable(with = WalletIdSerializer::class)
data class WalletId(
    val id: String,
)
