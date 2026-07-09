package com.gemwallet.android.features.asset.presents.chart

import androidx.compose.foundation.lazy.LazyListScope
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.perpetual.formatLeverage
import com.gemwallet.android.features.asset.viewmodels.chart.models.AllTimeUIModel
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.property.PnlPropertyItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PortfolioMarginUsage
import com.wallet.core.primitives.PortfolioStatistic

internal fun LazyListScope.portfolioStatistics(currency: Currency, statistics: List<PortfolioStatistic>) {
    if (statistics.isEmpty()) return
    item { SubheaderItem(R.string.common_info) }
    val (allTime, perpetual) = statistics.partition { it.asAllTimeUIModel() != null }
    allTimeProperties(currency, allTime.mapNotNull(PortfolioStatistic::asAllTimeUIModel))
    perpetualStatistics(currency, perpetual)
}

private fun LazyListScope.perpetualStatistics(currency: Currency, statistics: List<PortfolioStatistic>) {
    if (statistics.isEmpty()) return
    val currencyFormatter = CurrencyFormatter(currency = currency)
    val priceChangeFormatter = PriceChangeFormatter(currencyFormatter)
    itemsPositioned(statistics) { position, statistic ->
        when (statistic) {
            is PortfolioStatistic.UnrealizedPnl ->
                PnlPropertyItem(R.string.perpetual_unrealized_pnl, statistic.content, priceChangeFormatter, position)
            is PortfolioStatistic.AllTimePnl ->
                PnlPropertyItem(R.string.perpetual_all_time_pnl, statistic.content, priceChangeFormatter, position)
            is PortfolioStatistic.AccountLeverage ->
                PropertyItem(R.string.perpetual_account_leverage, statistic.content.formatLeverage(), listPosition = position)
            is PortfolioStatistic.MarginUsage ->
                PropertyItem(R.string.perpetual_margin_usage, statistic.content.marginText(currencyFormatter), listPosition = position)
            is PortfolioStatistic.Volume ->
                PropertyItem(R.string.perpetual_volume, currencyFormatter.string(statistic.content), listPosition = position)
            is PortfolioStatistic.AllTimeHigh, is PortfolioStatistic.AllTimeLow -> Unit
        }
    }
}

private fun PortfolioStatistic.asAllTimeUIModel(): AllTimeUIModel? = when (this) {
    is PortfolioStatistic.AllTimeHigh -> AllTimeUIModel.High(content.date, content.value.toDouble(), content.percentage.toDouble())
    is PortfolioStatistic.AllTimeLow -> AllTimeUIModel.Low(content.date, content.value.toDouble(), content.percentage.toDouble())
    else -> null
}

private fun PortfolioMarginUsage.marginText(formatter: CurrencyFormatter): String =
    "${formatter.string(accountValue * usage)} (${(usage * 100).formatAsPercentage(PercentageFormatterStyle.PercentSignLess)})"
