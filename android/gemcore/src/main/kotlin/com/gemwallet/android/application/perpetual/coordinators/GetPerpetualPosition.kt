package com.gemwallet.android.application.perpetual.coordinators

import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.wallet.core.primitives.PerpetualId
import kotlinx.coroutines.flow.Flow

interface GetPerpetualPosition {
    fun getPositionByPerpetual(id: PerpetualId): Flow<PerpetualPositionDetailsDataAggregate?>
}
