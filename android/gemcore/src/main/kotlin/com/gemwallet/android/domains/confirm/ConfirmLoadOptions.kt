package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoadOptions

fun FeeSelection.toGem(): GemConfirmFeeSelection = when (this) {
    is FeeSelection.Preset -> GemConfirmFeeSelection.Priority(priority.toGem())
    is FeeSelection.Custom -> GemConfirmFeeSelection.Custom(gasPrice)
}

fun confirmLoadOptions(selection: FeeSelection, feeAssetSelection: FeeAssetSelection, assetId: AssetId? = null) = GemConfirmLoadOptions(
    feeSelection = selection.toGem(),
    feeAssetId = when (feeAssetSelection) {
        FeeAssetSelection.Automatic -> null
        is FeeAssetSelection.Selected -> feeAssetSelection.assetId.toIdentifier()
    },
    assetId = assetId?.toIdentifier(),
)
