package com.gemwallet.android.domains.perpetual.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId
import uniffi.gemstone.GemPriceRow

interface PerpetualDataAggregate {

    val id: PerpetualId

    val title: String

    val price: GemPriceRow

    val volume: String

    val asset: Asset

    val isPinned: Boolean
}
