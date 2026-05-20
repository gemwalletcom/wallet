package com.gemwallet.android.model

import kotlinx.serialization.Serializable

@Serializable
data class DestinationAddress(
    val address: String,
    val name: String? = null,
) {
    companion object {
        val Hyperliquid = DestinationAddress(
            address = "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
            name = "Hyperliquid",
        )
    }
}