package com.gemwallet.android.features.perpetual.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPerpetualMarketSection

@StringRes
internal fun GemPerpetualMarketSection.stringRes(): Int? = when (this) {
    GemPerpetualMarketSection.POSITIONS -> R.string.perpetual_positions
    GemPerpetualMarketSection.PINNED -> R.string.common_pinned
    GemPerpetualMarketSection.MARKETS -> R.string.perpetuals_markets
    GemPerpetualMarketSection.RECENTS, GemPerpetualMarketSection.EMPTY -> null
}
