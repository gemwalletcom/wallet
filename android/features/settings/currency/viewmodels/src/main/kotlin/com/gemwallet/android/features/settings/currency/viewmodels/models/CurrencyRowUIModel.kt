package com.gemwallet.android.features.settings.currency.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.models.ListSection
import com.wallet.core.primitives.Currency
import uniffi.gemstone.GemCurrencyRow
import uniffi.gemstone.GemCurrencySection

data class CurrencyRowUIModel(val row: GemCurrencyRow, val model: ListItemModel)

internal fun GemCurrencySection.uiModel(context: Context): ListSection<CurrencyRowUIModel> = ListSection(
    id = kind.name,
    title = context.getString(kind.stringRes()),
    items = rows.map {
        CurrencyRowUIModel(row = it, model = ListItemModel(title = it.title))
    },
)
