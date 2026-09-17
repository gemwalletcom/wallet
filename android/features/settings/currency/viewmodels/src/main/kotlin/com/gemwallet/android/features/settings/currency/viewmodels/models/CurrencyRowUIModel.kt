package com.gemwallet.android.features.settings.currency.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.ListSection
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemCurrencies
import uniffi.gemstone.GemCurrencyRow

data class CurrencyRowUIModel(
    val currency: Currency,
    val isSelected: Boolean,
    val model: ListItemModel,
)

internal fun GemCurrencies.sections(context: Context): List<ListSection<CurrencyRowUIModel>> = listOf(
    ListSection(id = "recommended", title = context.getString(R.string.common_recommended), items = recommended.map { it.uiModel(selected) }),
    ListSection(id = "all", title = context.getString(R.string.common_all), items = other.map { it.uiModel(selected) }),
)

private fun GemCurrencyRow.uiModel(selected: GemCurrencyRow): CurrencyRowUIModel {
    val code = currency.toPrimitives().string
    return CurrencyRowUIModel(
        currency = currency.toPrimitives(),
        isSelected = currency == selected.currency,
        model = ListItemModel(title = "$flag  $code - ${android.icu.util.Currency.getInstance(code).displayName}"),
    )
}
