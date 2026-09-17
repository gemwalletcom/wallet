package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemAssetDetailsState
import uniffi.gemstone.GemSwapPairSuggestion

fun mockGemAssetDetails(
    asset: Asset = mockAsset(),
    state: GemAssetDetailsState = mockGemAssetDetailsState(),
) = GemAssetDetails(
    state = state,
    title = asset.name,
    explorerName = "Explorer",
    addressLink = null,
    tokenLink = null,
    verificationStatus = null,
    networkDestination = null,
    shareUrl = "",
    swapPair = GemSwapPairSuggestion(payAssetId = asset.id.toIdentifier(), receiveAssetId = null),
)
