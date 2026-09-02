package com.gemwallet.android.blockchain.services

import com.gemwallet.android.blockchain.gemstone.toFee
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.model.SignerParams
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmPreload

fun confirmLoadOptions(selection: FeeSelection, feeAssetSelection: FeeAssetSelection) = GemConfirmLoadOptions(
    feeSelection = when (selection) {
        is FeeSelection.Preset -> GemConfirmFeeSelection.Priority(selection.priority.toGem())
        is FeeSelection.Custom -> GemConfirmFeeSelection.Custom(selection.gasPrice.toString())
    },
    feeAssetId = when (feeAssetSelection) {
        FeeAssetSelection.Automatic -> null
        is FeeAssetSelection.Selected -> feeAssetSelection.assetId.toIdentifier()
    },
)

fun GemConfirmPreload.toSignerParams(input: GemConfirmInput): SignerParams {
    val selectedPriority = confirmData.selectedPriority.toPrimitives()
    return SignerParams(
        input = input,
        confirmData = confirmData,
        fee = confirmData.fee.toFee(selectedPriority, AssetId(confirmData.fee.feeAsset)),
        feeRates = confirmData.feeRates,
    )
}
