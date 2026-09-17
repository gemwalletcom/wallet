package com.gemwallet.android.testkit

import uniffi.gemstone.GemSignMessagePreview
import uniffi.gemstone.MessageType

fun mockGemSignMessagePreview(
    hasCriticalWarning: Boolean = false,
) = GemSignMessagePreview(
    messageType = MessageType.TEXT,
    text = "Sign in",
    primaryFields = emptyList(),
    secondaryFields = emptyList(),
    hasCriticalWarning = hasCriticalWarning,
    header = null,
)
