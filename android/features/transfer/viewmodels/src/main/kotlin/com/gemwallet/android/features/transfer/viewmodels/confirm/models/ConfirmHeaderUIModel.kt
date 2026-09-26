package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import android.content.Context
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.NftImageSource
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemTransactionHeader
import java.math.BigInteger

sealed interface ConfirmHeaderUIModel {
    data class Placeholder(val icon: Any?) : ConfirmHeaderUIModel
    data class ReservedSpace(val icon: Any?) : ConfirmHeaderUIModel
    data class Simulation(val header: SimulationHeaderUIModel) : ConfirmHeaderUIModel
    data class Swap(val fromAsset: Asset, val fromValueText: String, val fromEquivalentText: String?, val toAsset: Asset, val toValueText: String, val toEquivalentText: String?) : ConfirmHeaderUIModel
    data class Nft(val source: NftImageSource) : ConfirmHeaderUIModel
    data class Symbol(val asset: Asset) : ConfirmHeaderUIModel
    data class Amount(val amount: String, val equivalent: String?, val asset: Asset?) : ConfirmHeaderUIModel
}

data class FeeSelectionUIModel(val selectedPriority: FeePriority?, val customRate: BigInteger?)

internal fun confirmHeader(header: GemConfirmHeader, context: Context): ConfirmHeaderUIModel = when (header) {
    is GemConfirmHeader.Value -> ConfirmHeaderUIModel.Simulation(header.value.headerUIModel(context))
    is GemConfirmHeader.Placeholder -> ConfirmHeaderUIModel.Placeholder(header.assetId.toAssetId())
    is GemConfirmHeader.Reserved -> ConfirmHeaderUIModel.ReservedSpace(header.header.uiModel().headerIcon())
    is GemConfirmHeader.Transaction -> header.header.uiModel()
}

private fun ConfirmHeaderUIModel.headerIcon(): Any? = when (this) {
    is ConfirmHeaderUIModel.Amount -> asset
    is ConfirmHeaderUIModel.Symbol -> asset
    else -> null
}

private fun GemTransactionHeader.uiModel(): ConfirmHeaderUIModel = when (this) {
    is GemTransactionHeader.Swap -> ConfirmHeaderUIModel.Swap(
        fromAsset = from.asset.toPrimitives(),
        fromValueText = from.amount.text(),
        fromEquivalentText = from.fiat?.text(),
        toAsset = to.asset.toPrimitives(),
        toValueText = to.amount.text(),
        toEquivalentText = to.fiat?.text(),
    )

    is GemTransactionHeader.Nft -> ConfirmHeaderUIModel.Nft(NftImageSource(url = imageUrl, name = name.orEmpty()))

    is GemTransactionHeader.Symbol -> ConfirmHeaderUIModel.Symbol(asset.toPrimitives())

    is GemTransactionHeader.AssetImage -> ConfirmHeaderUIModel.Symbol(asset.toPrimitives())

    is GemTransactionHeader.Amount -> ConfirmHeaderUIModel.Amount(
        amount = amount.amount.text(),
        equivalent = amount.fiat?.text(),
        asset = amount.asset.toPrimitives(),
    )
}
