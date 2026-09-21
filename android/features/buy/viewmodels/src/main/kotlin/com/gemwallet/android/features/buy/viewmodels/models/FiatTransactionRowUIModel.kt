package com.gemwallet.android.features.buy.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.buy.localization.actionRes
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.iconResource
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.FiatTransactionAssetData
import uniffi.gemstone.fiatTransactionRow

data class FiatTransactionRowUIModel(val data: FiatTransactionAssetData, val model: ListItemModel)

internal fun FiatTransactionAssetData.uiModel(context: Context): FiatTransactionRowUIModel {
    val row = fiatTransactionRow(toGem())
    return FiatTransactionRowUIModel(
        data = this,
        model = ListItemModel(
            title = context.getString(row.quoteType.toPrimitives().actionRes()),
            titleTag = row.badge?.let { context.getString(it.stringRes()) },
            titleTagStyle = row.badge?.textStyle() ?: ListItemTextStyle.Secondary,
            titleExtra = row.subtitle,
            subtitle = row.value.text(),
            subtitleStyle = if (row.isDimmed) ListItemTextStyle.Secondary else ListItemTextStyle.Body,
            subtitleExtra = row.fiatValue.text(),
            image = ListItemImage.Drawable(row.provider.toPrimitives().iconResource()),
        ),
    )
}
