package com.gemwallet.android.features.transfer.viewmodels.confirm

import io.mockk.every
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmTitle
import uniffi.gemstone.GemConfirmViewState
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemKeystoreAuthentication

internal fun GemConfirmation.stubViewState(): GemConfirmation = apply {
    every { viewState(any()) } answers {
        val screen = firstArg<GemConfirmScreen>()
        GemConfirmViewState(screen.button(), screen.feeRow(), feeRateRows(), rowContents(null), emptyList(), GemConfirmTitle.Send, null, GemKeystoreAuthentication.NONE, null)
    }
}
