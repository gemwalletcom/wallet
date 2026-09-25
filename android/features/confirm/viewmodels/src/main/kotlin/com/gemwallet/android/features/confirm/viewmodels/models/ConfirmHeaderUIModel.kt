package com.gemwallet.android.features.confirm.viewmodels.models

import android.content.Context
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.ui.components.image.NftImageSource
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.components.list_head.amountText
import com.gemwallet.android.ui.components.list_head.fiat
import com.gemwallet.android.ui.components.list_head.headerUIModel
import com.gemwallet.android.ui.components.list_head.priceValue
import com.gemwallet.android.ui.components.list_head.valueText
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemTransactionHeader
import java.math.BigInteger

sealed interface ConfirmHeaderUIModel {
    data class Placeholder(val icon: Any?) : ConfirmHeaderUIModel
    data class ReservedSpace(val icon: Any?) : ConfirmHeaderUIModel
    data class Simulation(val header: SimulationHeaderUIModel) : ConfirmHeaderUIModel
    data class Swap(val fromAsset: AssetPriceValue, val fromValueText: String, val fromEquivalentText: String?, val toAsset: AssetPriceValue, val toValueText: String, val toEquivalentText: String?) : ConfirmHeaderUIModel
    data class Nft(val source: NftImageSource) : ConfirmHeaderUIModel
    data class Symbol(val asset: Asset) : ConfirmHeaderUIModel
    data class Amount(val amount: String, val equivalent: String?, val asset: Asset?) : ConfirmHeaderUIModel
}

data class FeeSelectionUIModel(val selectedPriority: FeePriority?, val customRate: BigInteger?)

internal fun confirmHeader(header: GemConfirmHeader, context: Context, currency: Currency): ConfirmHeaderUIModel = when (header) {
    is GemConfirmHeader.Value -> ConfirmHeaderUIModel.Simulation(header.value.headerUIModel(context))
    is GemConfirmHeader.Placeholder -> ConfirmHeaderUIModel.Placeholder(header.assetId.toAssetId())
    is GemConfirmHeader.Reserved -> ConfirmHeaderUIModel.ReservedSpace(header.header.uiModel(currency).headerIcon())
    is GemConfirmHeader.Transaction -> header.header.uiModel(currency)
}

private fun ConfirmHeaderUIModel.headerIcon(): Any? = when (this) {
    is ConfirmHeaderUIModel.Amount -> asset
    is ConfirmHeaderUIModel.Symbol -> asset
    else -> null
}

private fun GemTransactionHeader.uiModel(currency: Currency): ConfirmHeaderUIModel = when (this) {
    is GemTransactionHeader.Swap -> ConfirmHeaderUIModel.Swap(
        fromAsset = from.priceValue(currency),
        fromValueText = from.valueText(),
        fromEquivalentText = from.fiat(currency),
        toAsset = to.priceValue(currency),
        toValueText = to.valueText(),
        toEquivalentText = to.fiat(currency),
    )

    is GemTransactionHeader.Nft -> ConfirmHeaderUIModel.Nft(NftImageSource(url = imageUrl, name = name.orEmpty()))

    is GemTransactionHeader.Symbol -> ConfirmHeaderUIModel.Symbol(asset.toPrimitives())

    is GemTransactionHeader.AssetImage -> ConfirmHeaderUIModel.Symbol(asset.toPrimitives())

    is GemTransactionHeader.Amount -> ConfirmHeaderUIModel.Amount(
        amount = amount.amountText(),
        equivalent = amount.fiat(currency).takeIf { showsFiat },
        asset = amount.asset.toPrimitives(),
    )
}
