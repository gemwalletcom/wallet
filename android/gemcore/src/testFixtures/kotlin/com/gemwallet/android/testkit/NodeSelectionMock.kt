package com.gemwallet.android.testkit

import uniffi.gemstone.GemNodeSelection

fun mockGemNodeSelection(
    url: String = "https://node.example",
) = GemNodeSelection(
    url = url,
    host = url,
    isSelected = false,
    gemNodeFlag = null,
)
