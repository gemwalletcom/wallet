package com.gemwallet.android.testkit

import uniffi.gemstone.GemSwapAgain
import uniffi.gemstone.GemSwapProgress
import uniffi.gemstone.GemTransactionDetails

fun mockGemTransactionDetails(
    swapProgress: GemSwapProgress? = null,
    swapAgain: GemSwapAgain? = null,
    providerName: String? = null,
    estimatedConfirmationSeconds: UInt? = null,
    pnl: Double? = null,
    price: Double? = null,
) = GemTransactionDetails(
    swapProgress = swapProgress,
    swapAgain = swapAgain,
    providerName = providerName,
    estimatedConfirmationSeconds = estimatedConfirmationSeconds,
    pnl = pnl,
    price = price,
)
