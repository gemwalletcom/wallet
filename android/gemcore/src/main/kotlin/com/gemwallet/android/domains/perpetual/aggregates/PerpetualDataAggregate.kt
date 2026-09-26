package com.gemwallet.android.domains.perpetual.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId
import uniffi.gemstone.GemAssetItemRow

interface PerpetualDataAggregate {

    val id: PerpetualId

    val row: GemAssetItemRow

    val title: String get() = row.title

    val asset: Asset

    val isPinned: Boolean
}
