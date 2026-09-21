package com.gemwallet.android.testkit

import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemAssetDetailSection
import uniffi.gemstone.GemAssetDetails
import uniffi.gemstone.GemAssetDetailsState
import uniffi.gemstone.GemFormattedNumber
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemSwapPairSuggestion

fun mockGemAssetDetails(
    asset: Asset = mockAsset(),
    state: GemAssetDetailsState = mockGemAssetDetailsState(),
    sections: List<GemAssetDetailSection> = emptyList(),
    fiatValue: GemFormattedNumber? = null,
    balanceValue: GemFormattedNumber = mockFormattedNumber(value = 0.0, unit = GemNumberUnit.Symbol(symbol = asset.symbol)),
) = GemAssetDetails(
    state = state,
    balanceValue = balanceValue,
    sections = sections,
    title = asset.name,
    fiatValue = fiatValue,
    explorerName = "Explorer",
    addressLink = null,
    tokenLink = null,
    verificationStatus = null,
    networkDestination = null,
    shareUrl = "",
    swapPair = GemSwapPairSuggestion(payAssetId = asset.id.toIdentifier(), receiveAssetId = null),
)
