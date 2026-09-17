package com.gemwallet.android.features.buy.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.buy.localization.actionRes
import com.gemwallet.android.model.CurrencyFormatter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.ui.components.image.iconResource
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemTextStyle
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.style.textStyle
import com.wallet.core.primitives.FiatTransactionAssetData
import uniffi.gemstone.GemValueStyle
import uniffi.gemstone.fiatProviderName
import uniffi.gemstone.fiatTransactionStatus
import java.math.BigInteger

data class FiatTransactionRowUIModel(
    val data: FiatTransactionAssetData,
    val model: ListItemModel,
)

internal fun FiatTransactionAssetData.uiModel(context: Context): FiatTransactionRowUIModel {
    val status = fiatTransactionStatus(this.status.toGem())
    return FiatTransactionRowUIModel(
        data = this,
        model = ListItemModel(
            title = context.getString(transactionType.actionRes()),
            titleTag = status.badge?.let { context.getString(it.stringRes()) },
            titleTagStyle = status.badge?.textStyle() ?: ListItemTextStyle.Secondary,
            titleExtra = "${asset.name} (${fiatProviderName(provider.toGem())})",
            subtitle = ValueFormatter(style = GemValueStyle.SHORT).string(BigInteger(value), asset),
            subtitleStyle = if (status.isDimmed) ListItemTextStyle.Secondary else ListItemTextStyle.Body,
            subtitleExtra = CurrencyFormatter(type = CurrencyFormatter.Type.Fiat, currencyCode = fiatCurrency).string(fiatAmount),
            image = ListItemImage.Drawable(provider.iconResource()),
        ),
    )
}
