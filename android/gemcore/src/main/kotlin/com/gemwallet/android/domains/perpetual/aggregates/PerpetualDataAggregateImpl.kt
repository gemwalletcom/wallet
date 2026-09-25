package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.perpetualMarketRows

class PerpetualDataAggregateImpl(val data: PerpetualData, override val row: GemAssetItemRow) : PerpetualDataAggregate {

    override val id: PerpetualId = data.perpetual.id

    override val asset: Asset = data.asset

    override val isPinned: Boolean = data.metadata.isPinned
}

fun List<PerpetualData>.marketAggregates(): List<PerpetualDataAggregate> = zip(perpetualMarketRows(map { it.toGem() }), ::PerpetualDataAggregateImpl)
