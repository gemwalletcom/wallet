package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmMetadata
import uniffi.gemstone.GemConfirmPreload
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemFeeAsset

fun mockGemConfirmLoad(
    asset: Asset = mockAssetEthereum(),
    metadata: GemConfirmMetadata = mockGemConfirmMetadata(asset),
    feeAssets: List<GemFeeAsset> = emptyList(),
    simulation: GemConfirmSimulationState = GemConfirmSimulationState(
        chain = asset.id.chain.string,
        result = null,
        warnings = emptyList(),
        simulation = null,
        addressNames = emptyList(),
    ),
    addressName: String? = null,
    preload: GemConfirmPreload? = null,
) = GemConfirmLoad(
    feeAsset = asset.toGem(),
    metadata = metadata,
    feeAssets = feeAssets,
    simulation = simulation,
    addressName = addressName,
    preload = preload,
)
