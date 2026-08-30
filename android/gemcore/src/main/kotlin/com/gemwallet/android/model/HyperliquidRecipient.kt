package com.gemwallet.android.model

import com.gemwallet.android.domains.perpetual.PerpetualConfig
import uniffi.gemstone.GemRecipient

object HyperliquidRecipient {
    private const val NAME = "Hyperliquid"

    val provider: GemRecipient
        get() = GemRecipient(address = "", name = NAME)

    val deposit: GemRecipient
        get() = GemRecipient(address = PerpetualConfig.depositAddress, name = NAME)
}
