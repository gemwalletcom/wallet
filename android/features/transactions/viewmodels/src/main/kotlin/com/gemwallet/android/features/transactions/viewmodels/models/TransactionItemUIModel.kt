package com.gemwallet.android.features.transactions.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionParticipant

sealed interface TransactionItemUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : TransactionItemUIModel
    data class Address(val title: String, val address: String, val chain: Chain?, val text: String, val explorerLink: BlockExplorerLink?) : TransactionItemUIModel
    data class Fee(val model: ListItemModel) : TransactionItemUIModel
    data class SwapProgress(val model: TransactionSwapProgressUIModel) : TransactionItemUIModel
    data class Row(val row: GemListRow) : TransactionItemUIModel
    data class NftHead(val metadata: TransactionNFTTransferMetadata) : TransactionItemUIModel
    data class AmountHead(val asset: Asset, val amount: String, val equivalent: String?) : TransactionItemUIModel
    data class AssetHead(val asset: Asset) : TransactionItemUIModel
    data class SwapHead(val fromAsset: Asset, val fromValueText: String, val fromEquivalentText: String?, val toAsset: Asset, val toValueText: String, val toEquivalentText: String?) : TransactionItemUIModel

    data class SwapAgain(val fromAssetId: AssetId, val toAssetId: AssetId) : TransactionItemUIModel
}

fun GemTransactionDetailRows.chain(): Chain = asset.toPrimitives().chain

internal fun GemTransactionDetailRows.uiModel(row: GemTransactionDetailRow, context: Context): TransactionItemUIModel {
    val asset = asset.toPrimitives()
    return when (row) {
        GemTransactionDetailRow.Header -> header.head()

        GemTransactionDetailRow.SwapProgress -> TransactionItemUIModel.SwapProgress(requireNotNull(swapProgress).uiModel(context))

        GemTransactionDetailRow.SwapAgain -> requireNotNull(swapAgain).let {
            TransactionItemUIModel.SwapAgain(fromAssetId = AssetId(it.fromAssetId), toAssetId = AssetId(it.toAssetId))
        }

        GemTransactionDetailRow.Participant -> requireNotNull(participant).address(context, asset.chain)

        GemTransactionDetailRow.Fee -> TransactionItemUIModel.Fee(
            ListItemModel(
                title = feeRow.title.text(context),
                subtitle = feeRow.amount.text(),
                subtitleExtra = feeRow.fiat?.text(),
                info = feeRow.info.infoSheet(),
            ),
        )

        is GemTransactionDetailRow.Row -> TransactionItemUIModel.Row(row.row)
    }
}

private fun GemTransactionHeader.head(): TransactionItemUIModel = when (this) {
    is GemTransactionHeader.Amount -> TransactionItemUIModel.AmountHead(
        asset = amount.asset.toPrimitives(),
        amount = amount.amount.text(),
        equivalent = amount.fiat?.text().orEmpty(),
    )

    is GemTransactionHeader.Swap -> TransactionItemUIModel.SwapHead(
        fromAsset = from.asset.toPrimitives(),
        fromValueText = from.amount.text(),
        fromEquivalentText = from.fiat?.text(),
        toAsset = to.asset.toPrimitives(),
        toValueText = to.amount.text(),
        toEquivalentText = to.fiat?.text(),
    )

    is GemTransactionHeader.Nft -> TransactionItemUIModel.NftHead(
        TransactionNFTTransferMetadata(assetId = NFTAssetId(assetId), name = name),
    )

    is GemTransactionHeader.Symbol -> asset.toPrimitives().let { TransactionItemUIModel.AmountHead(it, it.symbol, null) }

    is GemTransactionHeader.AssetImage -> TransactionItemUIModel.AssetHead(asset.toPrimitives())
}

private fun GemTransactionParticipant.address(context: Context, chain: Chain): TransactionItemUIModel.Address = TransactionItemUIModel.Address(
    title = context.getString(role.stringRes()),
    address = address,
    chain = chain,
    text = text,
    explorerLink = link.toPrimitives(),
)
