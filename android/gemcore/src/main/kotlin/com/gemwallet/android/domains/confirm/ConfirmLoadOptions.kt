package com.gemwallet.android.domains.confirm

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.FeeAssetSelection
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoadOptions

fun confirmLoadOptions(selection: GemConfirmFeeSelection, feeAssetSelection: FeeAssetSelection) = GemConfirmLoadOptions(
    feeSelection = selection,
    feeAssetId = when (feeAssetSelection) {
        FeeAssetSelection.Automatic -> null
        is FeeAssetSelection.Selected -> feeAssetSelection.assetId.toIdentifier()
    },
)
