package com.gemwallet.android.features.activities.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatEstimatedConfirmation
import com.gemwallet.android.domains.transaction.values.TransactionDetailsValue
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.features.activities.viewmodels.localization.stringRes
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.infoSheet
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemListRow

sealed interface TransactionDetailsRowUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : TransactionDetailsRowUIModel
    data class Address(val title: String, val address: String, val chain: Chain?, val text: String, val explorerLink: BlockExplorerLink?) : TransactionDetailsRowUIModel
    data class Fee(val model: ListItemModel) : TransactionDetailsRowUIModel
    data class SwapProgress(val model: SwapProgressUIModel) : TransactionDetailsRowUIModel
    data class Value(val value: TransactionDetailsValue) : TransactionDetailsRowUIModel
    data class Row(val row: GemListRow, val infoIcon: Any?) : TransactionDetailsRowUIModel
}

internal fun TransactionDetailsValue.uiModel(context: Context, asset: Asset): TransactionDetailsRowUIModel = when (this) {
    is TransactionDetailsValue.EstimatedConfirmation -> TransactionDetailsRowUIModel.Item(
        ListItemModel(
            title = context.getString(R.string.transaction_estimated_confirmation),
            subtitle = formatEstimatedConfirmation(seconds),
            info = InfoSheetEntity.EstimatedConfirmationInfo(asset.chain),
        ),
    )

    is TransactionDetailsValue.Destination -> TransactionDetailsRowUIModel.Address(
        title = context.getString(stringRes()),
        address = data,
        chain = chain,
        text = text,
        explorerLink = explorerLink,
    )

    is TransactionDetailsValue.Fee -> TransactionDetailsRowUIModel.Fee(
        ListItemModel(
            title = row.title.text(context),
            subtitle = row.amount.text(),
            subtitleExtra = row.fiat?.text(),
            info = row.info.infoSheet(asset.iconModel()),
        ),
    )

    is TransactionDetailsValue.SwapProgress -> TransactionDetailsRowUIModel.SwapProgress(uiModel(context))

    is TransactionDetailsValue.Row -> TransactionDetailsRowUIModel.Row(row, asset.iconModel())

    is TransactionDetailsValue.Amount,
    is TransactionDetailsValue.Rate,
    is TransactionDetailsValue.SwapAgain,
    -> TransactionDetailsRowUIModel.Value(this)
}
