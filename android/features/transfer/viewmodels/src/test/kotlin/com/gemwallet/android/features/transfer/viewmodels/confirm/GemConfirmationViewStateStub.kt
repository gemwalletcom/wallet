package com.gemwallet.android.features.transfer.viewmodels.confirm

import io.mockk.every
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmSection
import uniffi.gemstone.GemConfirmStage
import uniffi.gemstone.GemConfirmTitle
import uniffi.gemstone.GemConfirmViewState
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemKeystoreAuthentication

internal fun GemConfirmation.stubViewState(): GemConfirmation = apply {
    every { viewState(any()) } answers {
        val screen = firstArg<GemConfirmScreen>()
        GemConfirmViewState(
            screen.button(),
            screen.feeRow(),
            feeRateRows(),
            GemConfirmTitle.Send,
            null,
            GemKeystoreAuthentication.NONE,
            listOfNotNull(
                GemConfirmSection.Header,
                GemConfirmSection.Details(rowContents(null)),
                GemConfirmSection.NetworkFee,
                screen.failure?.takeIf { it.stage == GemConfirmStage.LOAD }?.let { GemConfirmSection.Error(it.error) },
            ),
        )
    }
}
