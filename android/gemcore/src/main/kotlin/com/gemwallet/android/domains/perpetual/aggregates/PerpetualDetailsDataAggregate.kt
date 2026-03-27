package com.gemwallet.android.domains.perpetual.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider

interface PerpetualDetailsDataAggregate {
    val id: String
    val provider: PerpetualProvider
    val asset: Asset
    val name: String
    val dayVolume: String
    val openInterest: String
    val funding: String
    val maxLeverage: Int
    val price: Double
    val identifier: String
    val isIsolatedOnly: Boolean
    val marginType: PerpetualMarginType
        get() = if (isIsolatedOnly) PerpetualMarginType.Isolated else PerpetualMarginType.Cross
}
