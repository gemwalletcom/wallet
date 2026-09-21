package com.gemwallet.android.domains.perpetual.aggregates

import com.wallet.core.primitives.PerpetualPosition

interface PerpetualPositionDetailsDataAggregate : PerpetualPositionDataAggregate {
    val position: PerpetualPosition
}
