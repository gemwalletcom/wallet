package com.gemwallet.android.features.asset.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemAssetMarketRow

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
