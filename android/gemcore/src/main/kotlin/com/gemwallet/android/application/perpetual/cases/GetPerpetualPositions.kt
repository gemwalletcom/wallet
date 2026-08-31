package com.gemwallet.android.application.perpetual.cases

import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow

@OptIn(ExperimentalCoroutinesApi::class)
interface GetPerpetualPositions {

    fun getPerpetualPositions(): Flow<List<PerpetualPositionDataAggregate>>
}