package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualPositionData
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemPerpetualPositionRow
import uniffi.gemstone.perpetualPositionRows

class PerpetualPositionDataAggregateImpl(private val data: PerpetualPositionData, row: GemPerpetualPositionRow) : PerpetualPositionDataAggregate {
    override val perpetualId: PerpetualId
        get() = data.perpetual.id
    override val asset: Asset = data.asset
    override val row: GemAssetItemRow = row.row
}

fun List<PerpetualPositionData>.positionAggregates(): List<PerpetualPositionDataAggregate> = zip(perpetualPositionRows(map { it.toGem() }), ::PerpetualPositionDataAggregateImpl)
