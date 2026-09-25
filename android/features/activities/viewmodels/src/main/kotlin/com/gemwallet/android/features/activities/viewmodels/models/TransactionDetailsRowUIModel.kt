package com.gemwallet.android.features.activities.viewmodels.models

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

sealed interface TransactionDetailsRowUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : TransactionDetailsRowUIModel
    data class Address(val title: String, val address: String, val chain: Chain?, val text: String, val explorerLink: BlockExplorerLink?) : TransactionDetailsRowUIModel
    data class Fee(val model: ListItemModel) : TransactionDetailsRowUIModel
    data class SwapProgress(val model: SwapProgressUIModel) : TransactionDetailsRowUIModel
    data class Row(val row: GemListRow) : TransactionDetailsRowUIModel
    data class NftHead(val metadata: TransactionNFTTransferMetadata) : TransactionDetailsRowUIModel
    data class AmountHead(val asset: Asset, val amount: String, val equivalent: String?) : TransactionDetailsRowUIModel
    data class AssetHead(val asset: Asset) : TransactionDetailsRowUIModel
    data class SwapHead(val fromAsset: Asset, val fromValueText: String, val fromEquivalentText: String?, val toAsset: Asset, val toValueText: String, val toEquivalentText: String?) : TransactionDetailsRowUIModel

    data class SwapAgain(val fromAssetId: AssetId, val toAssetId: AssetId) : TransactionDetailsRowUIModel
}

fun GemTransactionDetailRows.chain(): Chain = asset.toPrimitives().chain

internal fun GemTransactionDetailRows.uiModel(row: GemTransactionDetailRow, context: Context): TransactionDetailsRowUIModel {
    val asset = asset.toPrimitives()
    return when (row) {
        GemTransactionDetailRow.Header -> header.head()

        GemTransactionDetailRow.SwapProgress -> TransactionDetailsRowUIModel.SwapProgress(requireNotNull(swapProgress).uiModel(context))

        GemTransactionDetailRow.SwapAgain -> requireNotNull(swapAgain).let {
            TransactionDetailsRowUIModel.SwapAgain(fromAssetId = AssetId(it.fromAssetId), toAssetId = AssetId(it.toAssetId))
        }

        GemTransactionDetailRow.Participant -> requireNotNull(participant).address(context, asset.chain)

        GemTransactionDetailRow.Fee -> TransactionDetailsRowUIModel.Fee(
            ListItemModel(
                title = feeRow.title.text(context),
                subtitle = feeRow.amount.text(),
                subtitleExtra = feeRow.fiat?.text(),
                info = feeRow.info.infoSheet(),
            ),
        )

        is GemTransactionDetailRow.Row -> TransactionDetailsRowUIModel.Row(row.row)
    }
}

private fun GemTransactionHeader.head(): TransactionDetailsRowUIModel = when (this) {
    is GemTransactionHeader.Amount -> TransactionDetailsRowUIModel.AmountHead(
        asset = amount.asset.toPrimitives(),
        amount = amount.amount.text(),
        equivalent = amount.fiat?.text().orEmpty(),
    )

    is GemTransactionHeader.Swap -> TransactionDetailsRowUIModel.SwapHead(
        fromAsset = from.asset.toPrimitives(),
        fromValueText = from.amount.text(),
        fromEquivalentText = from.fiat?.text(),
        toAsset = to.asset.toPrimitives(),
        toValueText = to.amount.text(),
        toEquivalentText = to.fiat?.text(),
    )

    is GemTransactionHeader.Nft -> TransactionDetailsRowUIModel.NftHead(
        TransactionNFTTransferMetadata(assetId = NFTAssetId(assetId), name = name),
    )

    is GemTransactionHeader.Symbol -> asset.toPrimitives().let { TransactionDetailsRowUIModel.AmountHead(it, it.symbol, null) }

    is GemTransactionHeader.AssetImage -> TransactionDetailsRowUIModel.AssetHead(asset.toPrimitives())
}

private fun GemTransactionParticipant.address(context: Context, chain: Chain): TransactionDetailsRowUIModel.Address = TransactionDetailsRowUIModel.Address(
    title = context.getString(role.stringRes()),
    address = address,
    chain = chain,
    text = text,
    explorerLink = link.toPrimitives(),
)
