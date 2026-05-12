package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPosition
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.format
import com.gemwallet.android.model.formatPnl
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualPositionData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class GetPerpetualPositionImpl @Inject constructor(
    private val perpetualRepository: PerpetualRepository
) : GetPerpetualPosition {
    override fun getPositionByPerpetual(id: String): Flow<PerpetualPositionDetailsDataAggregate?> {
        return perpetualRepository.getPositionByPerpetualId(id).map { PerpetualPositionDetailsDataAggregateImpl(it ?: return@map null) }
    }
}

class PerpetualPositionDetailsDataAggregateImpl(
    private val data: PerpetualPositionData,
) : PerpetualPositionDetailsDataAggregate {
    override val size: String = Currency.USD.format(data.position.sizeValue)

    override val entryPrice: String = Currency.USD.format(data.position.entryPrice, dynamicPlace = true)

    override val liquidationPrice: String = data.position.liquidationPrice
        ?.takeIf { it > 0.0 }
        ?.let { Currency.USD.format(it, dynamicPlace = true) }
        ?: ""

    override val marginType: PerpetualMarginType = data.position.marginType

    private val fundingPaymentsValue = data.position.funding?.toDouble()

    override val fundingPayments: String = fundingPaymentsValue?.let { Currency.USD.formatPnl(it, dynamicPlace = true) } ?: "-"

    override val fundingPaymentsDirection: ValueDirection = fundingPaymentsValue.toValueDirection()

    override val positionId: String = data.position.id

    override val perpetualId: String = data.position.perpetualId

    override val asset: Asset = data.asset

    override val name: String = data.perpetual.name

    override val direction: PerpetualDirection = data.position.direction

    override val leverage: Int = data.position.leverage.toInt()

    override val marginAmount: String = Currency.USD.format(data.position.marginAmount)

    override val entryValue: Double? = data.position.entryPrice

    override val liquidationValue: Double? = data.position.liquidationPrice?.takeIf { it > 0.0 }

    override val stopLoss: Double? = data.position.stopLoss?.price

    override val takeProfit: Double? = data.position.takeProfit?.price

    override val pnlWithPercentage: String
        get() = formatPnlWithPercentage(data.position.pnl, data.position.marginAmount)

    override val pnlState: ValueDirection
        get() = data.position.pnl.toValueDirection()
}
