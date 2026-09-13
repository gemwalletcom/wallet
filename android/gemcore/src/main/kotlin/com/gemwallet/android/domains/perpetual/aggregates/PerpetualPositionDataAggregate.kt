package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId

interface PerpetualPositionDataAggregate {
    val perpetualId: PerpetualId
    val asset: Asset
    val title: String
    val direction: PerpetualDirection
    val leverage: String
    val marginAmount: String
    val pnlWithPercentage: String
    val pnlState: ValueDirection
}
