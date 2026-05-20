package com.gemwallet.android.domains.perpetual.aggregates

import com.gemwallet.android.domains.price.values.EquivalentValue
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualId

interface PerpetualDataAggregate {

    val id: PerpetualId

    val name: String

    val price: EquivalentValue

    val volume: String

    val asset: Asset

    val isPinned: Boolean
}
