package com.gemwallet.android.domains.perpetual.aggregates

import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemValueTone

interface PerpetualPositionDataAggregate {
    val perpetualId: PerpetualId
    val asset: Asset
    val title: String
    val direction: PerpetualDirection
    val leverage: String
    val positionLabel: GemLocalizedText
    val marginAmount: String
    val pnl: GemLocalizedText
    val pnlState: GemValueTone
}
