package com.gemwallet.android.model

import com.gemwallet.android.domains.perpetual.PerpetualConfig
import kotlinx.serialization.Serializable

@Serializable
data class DestinationAddress(
    val address: String,
    val name: String? = null,
) {
    companion object {
        private const val HYPERLIQUID_NAME = "Hyperliquid"

        val hyperliquidProvider: DestinationAddress
            get() = DestinationAddress(
                address = "",
                name = HYPERLIQUID_NAME,
            )

        val hyperliquidDeposit: DestinationAddress
            get() = DestinationAddress(
                address = PerpetualConfig.depositAddress,
                name = HYPERLIQUID_NAME,
            )
    }
}
