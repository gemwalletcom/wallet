package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPositionData
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualPositionRow
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.perpetualPositionRows

class PerpetualPositionDataAggregateImpl(private val data: PerpetualPositionData, row: GemPerpetualPositionRow) : PerpetualPositionDataAggregate {
    override val perpetualId: PerpetualId
        get() = data.perpetual.id
    override val asset: Asset = data.asset

    override val title: String = row.title
    override val direction: PerpetualDirection = row.direction.toPrimitives()
    override val positionLabel: GemLocalizedText = row.position
    override val marginAmount: String = row.margin.text()
    override val pnl: GemLocalizedText = row.pnl
    override val pnlState: GemValueTone = row.pnlTone
}

fun List<PerpetualPositionData>.positionAggregates(): List<PerpetualPositionDataAggregate> = zip(perpetualPositionRows(map { it.toGem() }), ::PerpetualPositionDataAggregateImpl)
