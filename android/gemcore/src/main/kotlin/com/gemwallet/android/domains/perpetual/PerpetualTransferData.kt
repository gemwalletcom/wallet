package com.gemwallet.android.domains.perpetual

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import kotlinx.serialization.Serializable

@Serializable
data class PerpetualTransferData(
    val provider: PerpetualProvider,
    val direction: PerpetualDirection,
    val asset: Asset,
    val baseAsset: Asset,
    val assetIndex: Int,
    val price: Double,
    val leverage: UByte,
    val marginType: PerpetualMarginType,
)
