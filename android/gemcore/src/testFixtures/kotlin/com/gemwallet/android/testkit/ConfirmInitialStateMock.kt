package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.FeePriority
import uniffi.gemstone.GemConfirmInitialState
import uniffi.gemstone.GemConfirmSimulation

fun mockGemConfirmInitialState(
    asset: Asset = mockAssetEthereum(),
    feePriority: FeePriority = FeePriority.Normal,
    simulation: GemConfirmSimulation? = null,
) = GemConfirmInitialState(
    feePriority = feePriority.toGem(),
    feeAsset = asset.toGem(),
    simulation = simulation,
)
