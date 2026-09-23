package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.FeeAssetSelection
import com.wallet.core.primitives.AssetId
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoadOptions

fun confirmLoadOptions(selection: GemConfirmFeeSelection, feeAssetSelection: FeeAssetSelection, assetId: AssetId? = null) = GemConfirmLoadOptions(
    feeSelection = selection,
    feeAssetId = when (feeAssetSelection) {
        FeeAssetSelection.Automatic -> null
        is FeeAssetSelection.Selected -> feeAssetSelection.assetId.toIdentifier()
    },
    assetId = assetId?.toIdentifier(),
)
