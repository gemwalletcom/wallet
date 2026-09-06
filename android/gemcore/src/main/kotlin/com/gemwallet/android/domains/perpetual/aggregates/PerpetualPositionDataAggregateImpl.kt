package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.domains.perpetual.formatPnlWithPercentage
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.model.CurrencyFormatter
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPositionData

class PerpetualPositionDataAggregateImpl(
    private val data: PerpetualPositionData,
) : PerpetualPositionDataAggregate {
    private val marginFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

    override val positionId: String = data.position.id
    override val perpetualId: PerpetualId
        get() = data.perpetual.id
    override val asset: Asset = data.asset
    override val name: String = data.perpetual.name
    override val direction: PerpetualDirection = data.position.direction
    override val leverage: Int = data.position.leverage.toInt()
    override val marginAmount: String = marginFormatter.string(data.position.marginAmount)
    override val pnlWithPercentage: String = formatPnlWithPercentage(data.position.pnl, data.position.marginAmount)
    override val pnlState: ValueDirection = data.position.pnl.toValueDirection()
}
