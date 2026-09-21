package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.components.list_item.ListItemImageStyle

data class AcquireOptionUIModel(val action: AcquireAssetAction, val model: ListItemModel)

internal fun acquireOptions(context: Context, buyAmount: Int?): List<AcquireOptionUIModel> = listOf(
    AcquireOptionUIModel(
        action = AcquireAssetAction.Buy(buyAmount),
        model = ListItemModel(
            title = context.getString(R.string.wallet_buy),
            titleExtra = context.getString(R.string.wallet_pay_with_card_or_bank),
            image = ListItemImage.Symbol(ListItemSymbol.Buy, style = ListItemImageStyle.Action),
        ),
    ),
    AcquireOptionUIModel(
        action = AcquireAssetAction.Swap,
        model = ListItemModel(
            title = context.getString(R.string.wallet_swap),
            titleExtra = context.getString(R.string.wallet_from_your_wallet_assets),
            image = ListItemImage.Symbol(ListItemSymbol.Swap, style = ListItemImageStyle.Action),
        ),
    ),
    AcquireOptionUIModel(
        action = AcquireAssetAction.Receive,
        model = ListItemModel(
            title = context.getString(R.string.wallet_receive),
            titleExtra = context.getString(R.string.wallet_transfer_from_another_wallet),
            image = ListItemImage.Symbol(ListItemSymbol.Receive, style = ListItemImageStyle.Action),
        ),
    ),
)
