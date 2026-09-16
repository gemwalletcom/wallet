package com.gemwallet.android.testkit

import uniffi.gemstone.GemAssetRow
import uniffi.gemstone.GemAssetRowSubtitle
import uniffi.gemstone.GemAssetRowTitle
import uniffi.gemstone.GemAssetRowTrailing

fun mockGemAssetRow(
    title: GemAssetRowTitle = GemAssetRowTitle.ASSET,
) = GemAssetRow(
    title = title,
    showsSymbol = false,
    subtitle = GemAssetRowSubtitle.PRICE,
    trailing = GemAssetRowTrailing.BALANCE,
)
