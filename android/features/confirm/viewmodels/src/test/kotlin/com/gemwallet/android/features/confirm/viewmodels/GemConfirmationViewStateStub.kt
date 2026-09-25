package com.gemwallet.android.features.confirm.viewmodels

import io.mockk.every
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmViewState
import uniffi.gemstone.GemConfirmation

internal fun GemConfirmation.stubViewState(): GemConfirmation = apply {
    every { viewState(any(), any()) } answers {
        val screen = firstArg<GemConfirmScreen>()
        GemConfirmViewState(screen.button(), screen.feeRow(), feeRateRows(), rowContents(secondArg()), emptyList())
    }
}
