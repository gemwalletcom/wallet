package com.gemwallet.android.features.asset.viewmodels.localization

import uniffi.gemstone.GemPriceAlertToggle
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemAssetMarketRow
import uniffi.gemstone.PortfolioStatistic

@StringRes
internal fun GemAssetMarketRow.stringRes(): Int = when (this) {
    is GemAssetMarketRow.MarketCap -> R.string.asset_market_cap
    is GemAssetMarketRow.FullyDilutedValuation -> R.string.info_fully_diluted_valuation_title
    is GemAssetMarketRow.TradingVolume -> R.string.asset_trading_volume
    is GemAssetMarketRow.Contract -> R.string.asset_contract
    is GemAssetMarketRow.CirculatingSupply -> R.string.asset_circulating_supply
    is GemAssetMarketRow.TotalSupply -> R.string.asset_total_supply
    is GemAssetMarketRow.MaxSupply -> R.string.info_max_supply_title
    is GemAssetMarketRow.AllTimeHigh -> R.string.asset_all_time_high
    is GemAssetMarketRow.AllTimeLow -> R.string.asset_all_time_low
}

@StringRes
internal fun PortfolioStatistic.stringRes(): Int = when (this) {
    is PortfolioStatistic.AllTimeHigh -> R.string.asset_all_time_high
    is PortfolioStatistic.AllTimeLow -> R.string.asset_all_time_low
    is PortfolioStatistic.UnrealizedPnl -> R.string.perpetual_unrealized_pnl
    is PortfolioStatistic.AccountLeverage -> R.string.perpetual_account_leverage
    is PortfolioStatistic.MarginUsage -> R.string.perpetual_margin_usage
    is PortfolioStatistic.AllTimePnl -> R.string.perpetual_all_time_pnl
    is PortfolioStatistic.Volume -> R.string.perpetual_volume
}

@StringRes
fun GemPriceAlertToggle.toastRes(): Int = when (this) {
    GemPriceAlertToggle.ENABLED -> R.string.price_alerts_disabled_for
    GemPriceAlertToggle.DISABLED -> R.string.price_alerts_enabled_for
}
