package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmLoadOptions
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmScreen

fun mockGemConfirmLoadOptions(feeSelection: GemConfirmFeeSelection = GemConfirmFeeSelection.Priority(FeePriority.Normal.toGem()), feeAssetId: String? = null, assetId: String? = null) = GemConfirmLoadOptions(
    feeSelection = feeSelection,
    feeAssetId = feeAssetId,
    assetId = assetId,
)
