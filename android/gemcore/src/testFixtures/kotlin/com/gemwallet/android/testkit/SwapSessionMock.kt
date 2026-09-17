package com.gemwallet.android.testkit

import uniffi.gemstone.GemSwapQuotePhase
import uniffi.gemstone.GemSwapSession
import uniffi.gemstone.GemSwapTransferPhase

fun mockGemSwapSession() = GemSwapSession(
    quotePhase = GemSwapQuotePhase.NoInput,
    transferPhase = GemSwapTransferPhase.Idle,
)
