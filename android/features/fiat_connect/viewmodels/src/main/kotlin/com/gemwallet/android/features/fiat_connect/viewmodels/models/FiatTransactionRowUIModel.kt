package com.gemwallet.android.features.fiat_connect.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.iconResource
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.actionRes
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.FiatTransactionAssetData
import uniffi.gemstone.GemFiatTransactionRow
import uniffi.gemstone.fiatTransactionRows

data class FiatTransactionRowUIModel(val data: FiatTransactionAssetData, val model: ListItemModel)

internal fun List<FiatTransactionAssetData>.uiModels(context: Context): List<FiatTransactionRowUIModel> = zip(fiatTransactionRows(map { it.toGem() })) { data, row ->
    data.uiModel(row, context)
}

private fun FiatTransactionAssetData.uiModel(row: GemFiatTransactionRow, context: Context): FiatTransactionRowUIModel = FiatTransactionRowUIModel(
    data = this,
    model = ListItemModel(
        title = context.getString(row.quoteType.toPrimitives().actionRes()),
        titleTag = row.badge?.let { context.getString(it.stringRes()) },
        titleTagStyle = row.badge?.textStyle() ?: ListItemTextStyle.Secondary,
        titleExtra = row.subtitle,
        subtitle = row.value.text(),
        subtitleStyle = row.value.tone.textStyle(),
        subtitleExtra = row.fiatValue.text(),
        image = ListItemImage.Drawable(row.provider.toPrimitives().iconResource()),
    ),
)
