package com.gemwallet.android.testkit

import uniffi.gemstone.GemAssetRowStyle
import uniffi.gemstone.GemAssetSubtitleStyle
import uniffi.gemstone.GemAssetTitleStyle
import uniffi.gemstone.GemAssetTrailingStyle

fun mockGemAssetRowStyle(title: GemAssetTitleStyle = GemAssetTitleStyle.ASSET, showsSymbol: Boolean = false, subtitle: GemAssetSubtitleStyle = GemAssetSubtitleStyle.PRICE) = GemAssetRowStyle(
    title = title,
    showsSymbol = showsSymbol,
    subtitle = subtitle,
    trailing = GemAssetTrailingStyle.BALANCE,
)
