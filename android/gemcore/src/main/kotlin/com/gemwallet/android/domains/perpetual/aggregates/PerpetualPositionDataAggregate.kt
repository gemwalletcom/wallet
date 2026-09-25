package com.gemwallet.android.domains.perpetual.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId
import uniffi.gemstone.GemAssetItemRow

interface PerpetualPositionDataAggregate {
    val perpetualId: PerpetualId
    val asset: Asset
    val row: GemAssetItemRow
    val title: String get() = row.title
}
