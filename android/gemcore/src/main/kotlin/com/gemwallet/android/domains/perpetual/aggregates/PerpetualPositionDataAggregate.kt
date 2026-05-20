package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId

interface PerpetualPositionDataAggregate {
    val positionId: String
    val perpetualId: PerpetualId
    val asset: Asset
    val name: String
    val direction: PerpetualDirection
    val leverage: Int
    val marginAmount: String
    val pnlWithPercentage: String
    val pnlState: ValueDirection
}
