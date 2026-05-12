package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPositions
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualPositionData
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
class GetPerpetualPositionsImpl @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val perpetualRepository: PerpetualRepository,
) : GetPerpetualPositions {

    override fun getPerpetualPositions(): Flow<List<PerpetualPositionDataAggregateImpl>> {
        return sessionRepository.session()
            .filterNotNull()
            .flatMapLatest { perpetualRepository.getPositions(it.wallet.id) }
            .map { items -> items.map { PerpetualPositionDataAggregateImpl(it) } }
    }
}

class PerpetualPositionDataAggregateImpl(val data: PerpetualPositionData) : PerpetualPositionDataAggregate {
    override val positionId: String = data.position.id
    override val perpetualId: String = data.perpetual.id
    override val asset: Asset = data.asset
    override val name: String = data.perpetual.name
    override val direction: PerpetualDirection = data.position.direction
    override val leverage: Int = data.position.leverage.toInt()
    override val marginAmount: String = Currency.USD.format(data.position.marginAmount)
    override val pnlWithPercentage: String
        get() = formatPnlWithPercentage(data.position.pnl, data.position.marginAmount)
    override val pnlState: ValueDirection
        get() = data.position.pnl.toValueDirection()
}
