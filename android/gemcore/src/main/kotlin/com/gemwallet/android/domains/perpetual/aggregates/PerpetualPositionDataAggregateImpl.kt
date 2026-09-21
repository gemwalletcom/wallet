package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.domains.perpetual.formatPnlWithPercentage
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPositionData
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.perpetualPositionRow

class PerpetualPositionDataAggregateImpl(private val data: PerpetualPositionData) : PerpetualPositionDataAggregate {
    private val marginFormatter = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currency = Currency.USD)

    override val perpetualId: PerpetualId
        get() = data.perpetual.id
    override val asset: Asset = data.asset
    private val row = perpetualPositionRow(data.perpetual.toGem(), data.asset.toGem(), data.position.toGem())

    override val title: String = row.title
    override val direction: PerpetualDirection = data.position.direction
    override val leverage: String = row.leverage
    override val marginAmount: String = marginFormatter.string(data.position.marginAmount)
    override val pnlWithPercentage: String = formatPnlWithPercentage(data.position.pnl, data.position.marginAmount)
    override val pnlState: GemValueTone = data.position.pnl.tone()
}
