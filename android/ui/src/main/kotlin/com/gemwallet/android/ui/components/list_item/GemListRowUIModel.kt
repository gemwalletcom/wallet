package com.gemwallet.android.ui.components.list_item

import android.content.Context
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.titleRes
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle

internal sealed interface GemListRowUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : GemListRowUIModel
    data class Icon(val asset: Asset) : GemListRowUIModel
    data class Address(val address: String, val copy: GemCopy) : GemListRowUIModel
    data object Loading : GemListRowUIModel
}

internal fun GemListRow.uiModel(context: Context): GemListRowUIModel = when (this) {
    is GemListRow.Text -> GemListRowUIModel.Item(ListItemModel(title = context.getString(title.titleRes()), subtitle = value))
    is GemListRow.Amount -> GemListRowUIModel.Item(ListItemModel(title = context.getString(title.titleRes()), subtitle = amount.text()))
    is GemListRow.Explorer -> GemListRowUIModel.Item(ListItemModel(title = context.getString(R.string.transaction_view_on, name)), url = url)
    is GemListRow.Error -> GemListRowUIModel.Item(
        ListItemModel(title = context.getString(GemListRowTitle.ERROR.titleRes()), subtitle = error.message, titleStyle = ListItemTextStyle.Negative),
    )
    is GemListRow.Icon -> GemListRowUIModel.Icon(asset = chain.requireChain().asset())
    is GemListRow.Address -> GemListRowUIModel.Address(address = address, copy = copy)
    GemListRow.Loading -> GemListRowUIModel.Loading
}
