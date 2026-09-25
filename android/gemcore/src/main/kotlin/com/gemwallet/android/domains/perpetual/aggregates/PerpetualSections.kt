package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.PerpetualData
import uniffi.gemstone.perpetualMarketSections

data class PerpetualSections(val pinned: List<PerpetualDataAggregate> = emptyList(), val markets: List<PerpetualDataAggregate> = emptyList())

fun List<PerpetualData>.marketSections(): PerpetualSections {
    val sections = perpetualMarketSections(map { it.toGem() })
    return PerpetualSections(
        pinned = sections.pinned.map { PerpetualDataAggregateImpl(it.data.toPrimitives(), it.row) },
        markets = sections.markets.map { PerpetualDataAggregateImpl(it.data.toPrimitives(), it.row) },
    )
}
