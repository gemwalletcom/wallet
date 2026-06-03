package com.gemwallet.android.model

import com.wallet.core.primitives.Chain
import kotlinx.serialization.Serializable

@Serializable
data class ChainRecipient(
    val chain: Chain,
    val address: String,
    val memo: String?,
)
