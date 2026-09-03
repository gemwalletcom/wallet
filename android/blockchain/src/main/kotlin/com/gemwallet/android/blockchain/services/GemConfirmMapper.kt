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
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmPreload

fun FeeSelection.toGem(): GemConfirmFeeSelection = when (this) {
    is FeeSelection.Preset -> GemConfirmFeeSelection.Priority(priority.toGem())
    is FeeSelection.Custom -> GemConfirmFeeSelection.Custom(gasPrice)
}

fun confirmLoadOptions(selection: FeeSelection, feeAssetSelection: FeeAssetSelection) = GemConfirmLoadOptions(
    feeSelection = selection.toGem(),
    feeAssetId = when (feeAssetSelection) {
        FeeAssetSelection.Automatic -> null
        is FeeAssetSelection.Selected -> feeAssetSelection.assetId.toIdentifier()
    },
)

fun GemConfirmPreload.toSignerParams(): SignerParams {
    val selectedPriority = confirmData.selectedPriority.toPrimitives()
    return SignerParams(
        confirmData = confirmData,
        fee = confirmData.fee.toFee(selectedPriority, AssetId(confirmData.fee.feeAsset)),
    )
}
