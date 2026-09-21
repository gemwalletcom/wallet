package com.gemwallet.android.features.asset.viewmodels.chart.models

import android.content.Context
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.features.asset.viewmodels.localization.stringRes
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.PriceChangeFormatter
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemPercentageStyle
import uniffi.gemstone.PortfolioMarginUsage
import uniffi.gemstone.PortfolioStatistic
import uniffi.gemstone.leverageNumber

internal fun PortfolioStatistic.listItem(context: Context, currency: Currency): ListItemModel {
    val currencyFormatter = CurrencyFormatter(currency = currency)
    val priceChangeFormatter = PriceChangeFormatter(currencyFormatter)
    val title = context.getString(stringRes())
    return when (this) {
        is PortfolioStatistic.UnrealizedPnl -> ListItemModel(title = title, subtitle = priceChangeFormatter.string(value), subtitleStyle = value.tone().textStyle())
        is PortfolioStatistic.AllTimePnl -> ListItemModel(title = title, subtitle = priceChangeFormatter.string(value), subtitleStyle = value.tone().textStyle())
        is PortfolioStatistic.AccountLeverage -> ListItemModel(title = title, subtitle = leverageNumber(value).text())
        is PortfolioStatistic.MarginUsage -> ListItemModel(title = title, subtitle = value.marginText(currencyFormatter))
        is PortfolioStatistic.Volume -> ListItemModel(title = title, subtitle = currencyFormatter.string(value))
        is PortfolioStatistic.AllTimeHigh -> allTimeListItem(context, currency, true, value.date, value.value.toDouble(), value.percentage.toDouble())
        is PortfolioStatistic.AllTimeLow -> allTimeListItem(context, currency, false, value.date, value.value.toDouble(), value.percentage.toDouble())
    }
}

private fun PortfolioMarginUsage.marginText(formatter: CurrencyFormatter): String = "${formatter.string(usedValue)} (${usagePercent.formatAsPercentage(GemPercentageStyle.UNSIGNED)})"
