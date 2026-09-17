package com.gemwallet.android.testkit

import uniffi.gemstone.GemConnectionRow

fun mockGemConnectionRow(
    iconUrl: String? = mockApplicationMetadata().icon,
) = GemConnectionRow(
    title = "Uniswap",
    host = "app.uniswap.org",
    initial = "U",
    iconUrl = iconUrl,
)
