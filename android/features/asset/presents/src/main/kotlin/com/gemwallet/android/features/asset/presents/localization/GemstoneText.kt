package com.gemwallet.android.features.asset.presents.localization

import com.gemwallet.android.features.asset.viewmodels.chart.models.ChartSectionUIModel
import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.PortfolioType
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.PortfolioChartType
import uniffi.gemstone.PortfolioStatistic

@StringRes
internal fun PortfolioChartType.stringRes(): Int = when (this) {
    PortfolioChartType.VALUE -> R.string.perpetual_value
    PortfolioChartType.PNL -> R.string.perpetual_pnl
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
internal fun ChartSectionUIModel.stringRes(): Int? = when (this) {
    is ChartSectionUIModel.PriceAlerts -> R.string.settings_price_alerts_title
    ChartSectionUIModel.SetPriceAlert -> R.string.price_alerts_set_alert_title
    is ChartSectionUIModel.Links -> R.string.social_links
    is ChartSectionUIModel.Market -> null
}

@StringRes
internal fun PortfolioType.stringRes(): Int = when (this) {
    PortfolioType.Wallet -> R.string.wallet_portfolio_title
    PortfolioType.Perpetuals -> R.string.perpetuals_title
}

@StringRes
internal fun GemPriceAlertToggle.toastRes(): Int = when (this) {
    GemPriceAlertToggle.ENABLED -> R.string.price_alerts_disabled_for
    GemPriceAlertToggle.DISABLED -> R.string.price_alerts_enabled_for
}
