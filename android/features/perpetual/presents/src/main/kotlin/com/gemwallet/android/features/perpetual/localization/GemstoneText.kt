package com.gemwallet.android.features.perpetual.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemPerpetualMarketSection
import com.wallet.core.primitives.PerpetualMarginType
import uniffi.gemstone.GemPerpetualButton
import uniffi.gemstone.GemPerpetualInfoRow
import uniffi.gemstone.GemPerpetualPositionDetailRow
import uniffi.gemstone.GemPerpetualSection

@StringRes
internal fun PerpetualMarginType.stringRes(): Int = when (this) {
    PerpetualMarginType.Cross -> R.string.perpetual_margin_cross
    PerpetualMarginType.Isolated -> R.string.perpetual_margin_isolated
}

@StringRes
internal fun GemPerpetualSection.stringRes(): Int = when (this) {
    GemPerpetualSection.POSITION -> R.string.perpetual_position
    GemPerpetualSection.INFO -> R.string.common_info
}

@StringRes
internal fun GemPerpetualPositionDetailRow.stringRes(): Int = when (this) {
    GemPerpetualPositionDetailRow.PNL -> R.string.perpetual_pnl
    GemPerpetualPositionDetailRow.AUTOCLOSE -> R.string.perpetual_auto_close
    GemPerpetualPositionDetailRow.SIZE -> R.string.perpetual_size
    GemPerpetualPositionDetailRow.ENTRY_PRICE -> R.string.perpetual_entry_price
    GemPerpetualPositionDetailRow.LIQUIDATION_PRICE -> R.string.info_perpetual_liquidation_price_title
    GemPerpetualPositionDetailRow.MARGIN -> R.string.perpetual_margin
    GemPerpetualPositionDetailRow.FUNDING_PAYMENTS -> R.string.info_perpetual_funding_payments_title
}

@StringRes
internal fun GemPerpetualInfoRow.stringRes(): Int = when (this) {
    GemPerpetualInfoRow.DAILY_VOLUME -> R.string.markets_daily_volume
    GemPerpetualInfoRow.OPEN_INTEREST -> R.string.info_perpetual_open_interest_title
    GemPerpetualInfoRow.FUNDING_RATE -> R.string.info_perpetual_funding_apr_title
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
internal fun GemPerpetualMarketSection.stringRes(): Int? = when (this) {
    GemPerpetualMarketSection.POSITIONS -> R.string.perpetual_positions
    GemPerpetualMarketSection.PINNED -> R.string.common_pinned
    GemPerpetualMarketSection.MARKETS -> R.string.perpetuals_markets
    GemPerpetualMarketSection.RECENTS, GemPerpetualMarketSection.EMPTY -> null
}
