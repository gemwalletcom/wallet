package com.gemwallet.android.features.activities.viewmodels.models

import android.content.Context
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.duration.formatEstimatedConfirmation
import com.gemwallet.android.domains.swap.AssetRateFormatter
import com.gemwallet.android.domains.swap.AssetRatePair
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.activities.viewmodels.localization.stringRes
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.CryptoFiatConverter
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.InfoSheetEntity
import com.gemwallet.android.ui.components.image.iconModel
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.infoSheet
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemTransactionAmount
import uniffi.gemstone.GemTransactionDetailRow
import uniffi.gemstone.GemTransactionDetailRows
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransactionParticipant
import uniffi.gemstone.GemValueStyle

sealed interface TransactionDetailsRowUIModel {
    data class Item(val model: ListItemModel, val url: String? = null) : TransactionDetailsRowUIModel
    data class Address(val title: String, val address: String, val chain: Chain?, val text: String, val explorerLink: BlockExplorerLink?) : TransactionDetailsRowUIModel
    data class Fee(val model: ListItemModel) : TransactionDetailsRowUIModel
    data class SwapProgress(val model: SwapProgressUIModel) : TransactionDetailsRowUIModel
    data class Row(val row: GemListRow, val infoIcon: Any?) : TransactionDetailsRowUIModel
    data class NftHead(val metadata: TransactionNFTTransferMetadata) : TransactionDetailsRowUIModel
    data class AmountHead(val asset: Asset, val amount: String, val equivalent: String?) : TransactionDetailsRowUIModel
    data class SwapHead(val fromAsset: AssetPriceValue, val fromValueText: String, val fromEquivalentText: String?, val toAsset: AssetPriceValue, val toValueText: String, val toEquivalentText: String?) : TransactionDetailsRowUIModel

    data class Rate(val rate: AssetRatePair) : TransactionDetailsRowUIModel
    data class SwapAgain(val fromAssetId: AssetId, val toAssetId: AssetId) : TransactionDetailsRowUIModel
}

internal fun GemTransactionDetailRows.uiModel(row: GemTransactionDetailRow, context: Context, currency: Currency): TransactionDetailsRowUIModel {
    val asset = asset.toPrimitives()
    return when (row) {
        GemTransactionDetailRow.Header -> header.head(currency)

        GemTransactionDetailRow.SwapProgress -> TransactionDetailsRowUIModel.SwapProgress(requireNotNull(swapProgress).uiModel(context))

        GemTransactionDetailRow.SwapAgain -> requireNotNull(swapAgain).let {
            TransactionDetailsRowUIModel.SwapAgain(fromAssetId = AssetId(it.fromAssetId), toAssetId = AssetId(it.toAssetId))
        }

        GemTransactionDetailRow.EstimatedConfirmation -> TransactionDetailsRowUIModel.Item(
            ListItemModel(
                title = context.getString(R.string.transaction_estimated_confirmation),
                subtitle = formatEstimatedConfirmation(requireNotNull(estimatedConfirmationSeconds)),
                info = InfoSheetEntity.EstimatedConfirmationInfo(asset.chain),
            ),
        )

        GemTransactionDetailRow.Participant -> requireNotNull(participant).address(context, asset.chain)

        GemTransactionDetailRow.Rate -> TransactionDetailsRowUIModel.Rate(AssetRateFormatter().format(requireNotNull(rate)))

        GemTransactionDetailRow.Fee -> TransactionDetailsRowUIModel.Fee(
            ListItemModel(
                title = feeRow.title.text(context),
                subtitle = feeRow.amount.text(),
                subtitleExtra = feeRow.fiat?.text(),
                info = feeRow.info.infoSheet(context, asset.iconModel()),
            ),
        )

        is GemTransactionDetailRow.Row -> TransactionDetailsRowUIModel.Row(row.row, asset.iconModel())
    }
}

private fun GemTransactionHeader.head(currency: Currency): TransactionDetailsRowUIModel = when (this) {
    is GemTransactionHeader.Amount -> amount.head(currency, showsFiat)

    is GemTransactionHeader.Swap -> TransactionDetailsRowUIModel.SwapHead(
        fromAsset = from.priceValue(currency),
        fromValueText = from.valueText(),
        fromEquivalentText = from.fiat(currency),
        toAsset = to.priceValue(currency),
        toValueText = to.valueText(),
        toEquivalentText = to.fiat(currency),
    )

    is GemTransactionHeader.Nft -> TransactionDetailsRowUIModel.NftHead(
        TransactionNFTTransferMetadata(assetId = NFTAssetId(assetId), name = name),
    )

    is GemTransactionHeader.Symbol -> asset.toPrimitives().let { TransactionDetailsRowUIModel.AmountHead(it, it.symbol, null) }

    is GemTransactionHeader.AssetImage -> asset.toPrimitives().let { TransactionDetailsRowUIModel.AmountHead(it, it.symbol, null) }
}

private fun GemTransactionAmount.head(currency: Currency, showsFiat: Boolean): TransactionDetailsRowUIModel.AmountHead {
    val asset = asset.toPrimitives()
    return TransactionDetailsRowUIModel.AmountHead(
        asset = asset,
        amount = sign.amount(value, asset.decimals.toUInt(), asset.symbol, GemValueStyle.AUTO).text(),
        equivalent = fiat(currency).takeIf { showsFiat }.orEmpty(),
    )
}

private fun GemTransactionAmount.valueText(): String = ValueFormatter(style = GemValueStyle.AUTO).string(value, asset.toPrimitives())

private fun GemTransactionAmount.fiat(currency: Currency): String? = price?.let {
    CryptoFiatConverter.toFiatString(Crypto(value), asset.toPrimitives().decimals, it.price, currency)
}

private fun GemTransactionAmount.priceValue(currency: Currency): AssetPriceValue = AssetPriceValue(
    asset = asset.toPrimitives(),
    price = price?.let { AssetPriceInfo(currency, it.toPrimitives()) },
)

private fun GemTransactionParticipant.address(context: Context, chain: Chain): TransactionDetailsRowUIModel.Address = TransactionDetailsRowUIModel.Address(
    title = context.getString(role.stringRes()),
    address = address,
    chain = chain,
    text = text,
    explorerLink = link.toPrimitives(),
)
