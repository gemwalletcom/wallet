package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmPreload

fun mockGemConfirmLoad(
    asset: Asset = mockAssetEthereum(),
    preload: GemConfirmPreload? = null,
) = GemConfirmLoad(
    sender = mockAccount(chain = asset.id.chain).toGem(),
    feeAsset = asset.toGem(),
    metadata = mockGemConfirmMetadata(asset),
    feeAssets = emptyList(),
    simulation = mockGemConfirmSimulationState(chain = asset.id.chain),
    addressName = null,
    preload = preload,
)
