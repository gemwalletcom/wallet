package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPosition
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregate
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDataAggregateImpl
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualPositionDetailsDataAggregate
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualPositionData
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import javax.inject.Inject

class GetPerpetualPositionImpl @Inject constructor(
    private val perpetualRepository: PerpetualRepository
) : GetPerpetualPosition {
    override fun getPositionByPerpetual(id: PerpetualId): Flow<PerpetualPositionDetailsDataAggregate?> {
        return perpetualRepository.getPositionByPerpetualId(id).map { PerpetualPositionDetailsDataAggregateImpl(it ?: return@map null) }
    }
}

class PerpetualPositionDetailsDataAggregateImpl(
    private val data: PerpetualPositionData,
) : PerpetualPositionDetailsDataAggregate,
    PerpetualPositionDataAggregate by PerpetualPositionDataAggregateImpl(data) {

    private val amountFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)
    private val priceFormatter = CurrencyFormatter(currency = Currency.USD)

    override val size: String = amountFormatter.string(data.position.sizeValue)

    override val entryPrice: String = priceFormatter.string(data.position.entryPrice)

    override val liquidationPrice: String = data.position.liquidationPrice
        ?.takeIf { it > 0.0 }
        ?.let { priceFormatter.string(it) }
        ?: ""

    override val marginType: PerpetualMarginType = data.position.marginType

    private val fundingPaymentsValue = data.position.funding?.toDouble()

    override val fundingPayments: String = fundingPaymentsValue
        ?.let { PriceChangeFormatter(priceFormatter).string(it) }
        ?: "-"

    override val fundingPaymentsDirection: ValueDirection = fundingPaymentsValue.toValueDirection()

    override val perpetualId: PerpetualId = data.position.perpetualId

    override val entryValue: Double? = data.position.entryPrice

    override val liquidationValue: Double? = data.position.liquidationPrice?.takeIf { it > 0.0 }

    override val stopLoss: Double? = data.position.stopLoss?.price

    override val takeProfit: Double? = data.position.takeProfit?.price
}
