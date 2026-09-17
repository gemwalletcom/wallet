package com.gemwallet.android.testkit

import uniffi.gemstone.GemAssetRowStyle
import uniffi.gemstone.GemAssetSubtitleStyle
import uniffi.gemstone.GemAssetTitleStyle
import uniffi.gemstone.GemAssetTrailingStyle

fun mockGemAssetRowStyle(
    title: GemAssetTitleStyle = GemAssetTitleStyle.ASSET,
) = GemAssetRowStyle(
    title = title,
    showsSymbol = false,
    subtitle = GemAssetSubtitleStyle.PRICE,
    trailing = GemAssetTrailingStyle.BALANCE,
)
