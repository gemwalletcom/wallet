package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.cases.GetPerpetualPositions
import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregateImpl
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class GetPerpetualPositionsImpl @Inject constructor(
    private val getSession: GetSession,
    private val perpetualStore: GemstonePerpetualStore,
) : GetPerpetualPositions {

    override fun getPerpetualPositions(): Flow<List<PerpetualPositionDataAggregate>> {
        return getSession()
            .filterNotNull()
            .flatMapLatest { perpetualStore.observePositions(it.wallet.id) }
            .map { items -> items.map { PerpetualPositionDataAggregateImpl(it) } }
    }
}
