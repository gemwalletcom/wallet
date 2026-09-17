package com.gemwallet.android.testkit

import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmScreen

fun mockGemConfirmScreen() = GemConfirmScreen(
    phase = GemConfirmPhase.LOADING,
    hasCriticalWarning = false,
    failure = null,
)
