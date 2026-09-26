package com.gemwallet.android.features.transactions.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemAddressRow
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionHeader

sealed interface TransactionItemUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : TransactionItemUIModel
    data class Address(val row: GemAddressRow) : TransactionItemUIModel
    data class Fee(val model: ListItemModel, val details: ListItemModel) : TransactionItemUIModel
    data class SwapProgress(val model: TransactionSwapProgressUIModel) : TransactionItemUIModel
    data class Row(val row: GemListRow) : TransactionItemUIModel
    data class Head(val header: GemTransactionHeader) : TransactionItemUIModel
    data class SwapAgain(val fromAssetId: AssetId, val toAssetId: AssetId) : TransactionItemUIModel
}

fun GemTransactionDetailRows.chain(): Chain = asset.toPrimitives().chain

internal fun GemTransactionDetailRows.uiModel(row: GemTransactionDetailRow, context: Context): TransactionItemUIModel {
    val asset = asset.toPrimitives()
    return when (row) {
        GemTransactionDetailRow.Header -> TransactionItemUIModel.Head(header)

        GemTransactionDetailRow.SwapProgress -> TransactionItemUIModel.SwapProgress(requireNotNull(swapProgress).uiModel(context))

        GemTransactionDetailRow.SwapAgain -> requireNotNull(swapAgain).let {
            TransactionItemUIModel.SwapAgain(fromAssetId = AssetId(it.fromAssetId), toAssetId = AssetId(it.toAssetId))
        }

        is GemTransactionDetailRow.Participant -> TransactionItemUIModel.Address(row.row)

        GemTransactionDetailRow.Fee -> TransactionItemUIModel.Fee(
            model = ListItemModel(
                title = feeRow.title.text(context),
                subtitle = feeRow.text.value.text(),
                subtitleExtra = feeRow.text.extra?.string(context),
                info = feeRow.info.infoSheet(),
            ),
            details = ListItemModel(
                title = feeRow.title.text(context),
                subtitle = feeRow.fee.amount.text(),
                subtitleExtra = feeRow.fee.fiat?.text(),
            ),
        )

        is GemTransactionDetailRow.Row -> TransactionItemUIModel.Row(row.row)
    }
}
