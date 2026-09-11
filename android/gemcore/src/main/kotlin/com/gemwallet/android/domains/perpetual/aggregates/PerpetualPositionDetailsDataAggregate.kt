package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualPosition

interface PerpetualPositionDetailsDataAggregate : PerpetualPositionDataAggregate {
    val size: String
    val entryPrice: String
    val liquidationPrice: String
    val marginType: PerpetualMarginType
    val fundingPayments: String
    val fundingPaymentsDirection: ValueDirection
    val stopLoss: Double?
    val takeProfit: Double?
    val position: PerpetualPosition
}
