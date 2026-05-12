package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.domains.price.ValueDirection
import com.wallet.core.primitives.PerpetualMarginType

interface PerpetualPositionDetailsDataAggregate : PerpetualPositionDataAggregate {
    val size: String
    val entryPrice: String
    val liquidationPrice: String
    val marginType: PerpetualMarginType
    val fundingPayments: String
    val fundingPaymentsDirection: ValueDirection
    val entryValue: Double?
    val liquidationValue: Double?
    val stopLoss: Double?
    val takeProfit: Double?
}
