package com.gemwallet.android.features.asset.viewmodels.chart.models

import android.content.Context
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.tone
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.Currency
import java.text.DateFormat
import java.util.Date

internal fun allTimeListItem(context: Context, currency: Currency, isHigh: Boolean, date: Long, value: Double, percentage: Double): ListItemModel = ListItemModel(
    title = context.getString(if (isHigh) R.string.asset_all_time_high else R.string.asset_all_time_low),
    titleExtra = DateFormat.getDateInstance(DateFormat.MEDIUM).format(Date(date)),
    subtitle = CurrencyFormatter(currency = currency).string(value),
    subtitleExtra = percentage.formatAsPercentage(),
    subtitleExtraStyle = percentage.tone().textStyle(),
)
