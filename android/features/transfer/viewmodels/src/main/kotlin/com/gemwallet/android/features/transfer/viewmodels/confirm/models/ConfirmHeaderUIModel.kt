package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import android.content.Context
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.text
import com.gemwallet.android.ui.components.image.NftImageSource
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemAssetIcon
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemValueHeader
import java.math.BigInteger

sealed interface ConfirmHeaderUIModel {
    data class Placeholder(val icon: Any?) : ConfirmHeaderUIModel
    data class ReservedSpace(val icon: Any?) : ConfirmHeaderUIModel
    data class Simulation(val header: GemValueHeader) : ConfirmHeaderUIModel
    data class Swap(val fromAsset: Asset, val fromIcon: GemAssetIcon, val fromValueText: String, val fromEquivalentText: String?, val toAsset: Asset, val toIcon: GemAssetIcon, val toValueText: String, val toEquivalentText: String?) :
        ConfirmHeaderUIModel
    data class Nft(val source: NftImageSource) : ConfirmHeaderUIModel
    data class Symbol(val asset: Asset, val icon: GemAssetIcon) : ConfirmHeaderUIModel
    data class AssetImage(val icon: GemAssetIcon) : ConfirmHeaderUIModel
    data class Amount(val amount: String, val equivalent: String?, val asset: Asset?, val icon: GemAssetIcon?) : ConfirmHeaderUIModel
}

data class FeeSelectionUIModel(val selectedPriority: FeePriority?, val customRate: BigInteger?)

internal fun confirmHeader(header: GemConfirmHeader, context: Context): ConfirmHeaderUIModel = when (header) {
    is GemConfirmHeader.Value -> ConfirmHeaderUIModel.Simulation(header.value.header)
    is GemConfirmHeader.Placeholder -> ConfirmHeaderUIModel.Placeholder(header.icon)
    is GemConfirmHeader.Reserved -> ConfirmHeaderUIModel.ReservedSpace(header.header.uiModel().headerIcon())
    is GemConfirmHeader.Transaction -> header.header.uiModel()
}

private fun ConfirmHeaderUIModel.headerIcon(): Any? = when (this) {
    is ConfirmHeaderUIModel.Amount -> icon
    is ConfirmHeaderUIModel.Symbol -> icon
    is ConfirmHeaderUIModel.AssetImage -> icon
    else -> null
}

private fun GemTransactionHeader.uiModel(): ConfirmHeaderUIModel = when (this) {
    is GemTransactionHeader.Swap -> ConfirmHeaderUIModel.Swap(
        fromAsset = from.asset.toPrimitives(),
        fromIcon = from.icon,
        fromValueText = from.amount.text(),
        fromEquivalentText = from.fiat?.text(),
        toAsset = to.asset.toPrimitives(),
        toIcon = to.icon,
        toValueText = to.amount.text(),
        toEquivalentText = to.fiat?.text(),
    )

    is GemTransactionHeader.Nft -> ConfirmHeaderUIModel.Nft(NftImageSource(url = imageUrl, name = name.orEmpty()))

    is GemTransactionHeader.Symbol -> ConfirmHeaderUIModel.Symbol(asset.toPrimitives(), icon)

    is GemTransactionHeader.AssetImage -> ConfirmHeaderUIModel.AssetImage(icon)

    is GemTransactionHeader.Amount -> ConfirmHeaderUIModel.Amount(
        amount = amount.amount.text(),
        equivalent = amount.fiat?.text(),
        asset = amount.asset.toPrimitives(),
        icon = amount.icon,
    )
}
