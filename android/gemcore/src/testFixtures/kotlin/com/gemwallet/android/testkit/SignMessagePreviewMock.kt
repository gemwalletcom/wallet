package com.gemwallet.android.testkit

import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemSignMessagePreview

fun mockGemSignMessagePreview(hasCriticalWarning: Boolean = false) = GemSignMessagePreview(
    title = GemLocalizedText.ReviewRequest,
    text = "Sign in",
    primaryFields = emptyList(),
    secondaryFields = emptyList(),
    hasCriticalWarning = hasCriticalWarning,
    header = null,
    rows = emptyList(),
    warnings = emptyList(),
)
