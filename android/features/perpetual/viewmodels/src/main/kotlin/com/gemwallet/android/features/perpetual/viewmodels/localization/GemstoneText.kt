package com.gemwallet.android.features.perpetual.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualChartLineKind
import uniffi.gemstone.GemPerpetualMarketSection
import uniffi.gemstone.GemPerpetualSection

@StringRes
internal fun GemPerpetualSection.stringRes(): Int = when (this) {
    is GemPerpetualSection.Position -> R.string.perpetual_position
    is GemPerpetualSection.Info -> R.string.common_info
}

@StringRes
internal fun GemPerpetualButton.stringRes(): Int = when (this) {
    GemPerpetualButton.LONG -> R.string.perpetual_long
    GemPerpetualButton.SHORT -> R.string.perpetual_short
    GemPerpetualButton.MODIFY -> R.string.perpetual_modify
    GemPerpetualButton.CLOSE -> R.string.perpetual_close_position
    GemPerpetualButton.INCREASE -> R.string.perpetual_increase_position
    GemPerpetualButton.REDUCE -> R.string.perpetual_reduce_position
}

@StringRes
fun GemPerpetualMarketSection.stringRes(): Int? = when (this) {
    GemPerpetualMarketSection.POSITIONS -> R.string.perpetual_positions
    GemPerpetualMarketSection.PINNED -> R.string.common_pinned
    GemPerpetualMarketSection.MARKETS -> R.string.perpetuals_markets
    GemPerpetualMarketSection.RECENTS, GemPerpetualMarketSection.EMPTY -> null
}

@StringRes
fun GemPerpetualChartLineKind.stringRes(): Int = when (this) {
    GemPerpetualChartLineKind.ENTRY -> R.string.charts_entry
    GemPerpetualChartLineKind.LIQUIDATION -> R.string.perpetual_liquidation
    GemPerpetualChartLineKind.STOP_LOSS -> R.string.perpetual_stop_loss
    GemPerpetualChartLineKind.TAKE_PROFIT -> R.string.perpetual_take_profit
}
