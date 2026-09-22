package com.gemwallet.android.features.confirm.viewmodels.models

import com.gemwallet.android.domains.confirm.AmountUIModel
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.ui.components.list_head.SimulationHeaderUIModel
import com.gemwallet.android.ui.models.ButtonState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.NFTAsset
import uniffi.gemstone.GemConfirmButtonState
import uniffi.gemstone.GemTransactionHeaderKind
import java.math.BigInteger

sealed interface ConfirmHeaderUIModel {
    data class Placeholder(val asset: Asset?) : ConfirmHeaderUIModel
    data class Simulation(val header: SimulationHeaderUIModel) : ConfirmHeaderUIModel
    data class Swap(val fromAsset: AssetPriceValue, val fromValueText: String, val fromEquivalentText: String?, val toAsset: AssetPriceValue, val toValueText: String, val toEquivalentText: String?) : ConfirmHeaderUIModel
    data class Nft(val nftAsset: NFTAsset) : ConfirmHeaderUIModel
    data class Symbol(val asset: Asset) : ConfirmHeaderUIModel
    data class Amount(val amount: String, val equivalent: String?, val asset: Asset?) : ConfirmHeaderUIModel
}

data class FeeSelectionUIModel(val selectedPriority: FeePriority?, val customRate: BigInteger?)

internal fun confirmHeader(amountModel: AmountUIModel?, simulationHeader: SimulationHeaderUIModel?, isPlaceholder: Boolean, isLoading: Boolean, headerAsset: Asset?): ConfirmHeaderUIModel? = when {
    isPlaceholder && simulationHeader == null && isLoading -> ConfirmHeaderUIModel.Placeholder(headerAsset)

    simulationHeader != null -> ConfirmHeaderUIModel.Simulation(simulationHeader)

    amountModel?.headerKind is GemTransactionHeaderKind.Swap -> ConfirmHeaderUIModel.Swap(
        fromAsset = amountModel.fromAsset,
        fromValueText = amountModel.fromAmountText,
        fromEquivalentText = amountModel.fromAmountEquivalentText,
        toAsset = requireNotNull(amountModel.toAsset),
        toValueText = requireNotNull(amountModel.toAmountText),
        toEquivalentText = amountModel.toAmountEquivalentText,
    )

    amountModel?.headerKind is GemTransactionHeaderKind.Nft -> amountModel.nftAsset?.let { ConfirmHeaderUIModel.Nft(it) }

    amountModel?.headerKind is GemTransactionHeaderKind.Symbol || amountModel?.headerKind is GemTransactionHeaderKind.AssetImage -> ConfirmHeaderUIModel.Symbol(amountModel.asset)

    amountModel?.headerKind is GemTransactionHeaderKind.Payment -> ConfirmHeaderUIModel.Amount(
        amount = amountModel.fromAmountText,
        equivalent = amountModel.amountEquivalent,
        asset = headerAsset,
    )

    else -> ConfirmHeaderUIModel.Amount(
        amount = amountModel?.cryptoAmount ?: "",
        equivalent = amountModel?.amountEquivalent?.takeIf { (amountModel.headerKind as? GemTransactionHeaderKind.Amount)?.showsFiat != false },
        asset = headerAsset,
    )
}

internal fun GemConfirmButtonState.buttonState(): ButtonState = when (this) {
    GemConfirmButtonState.DISABLED -> ButtonState.Disabled
    GemConfirmButtonState.LOADING -> ButtonState.Loading
    GemConfirmButtonState.ENABLED -> ButtonState.Enabled
}
