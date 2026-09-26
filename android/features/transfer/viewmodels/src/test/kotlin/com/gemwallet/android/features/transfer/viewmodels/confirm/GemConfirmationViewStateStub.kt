package com.gemwallet.android.features.transfer.viewmodels.confirm

import com.gemwallet.android.testkit.mockGemConfirmFeeRow
import com.gemwallet.android.testkit.mockGemConfirmHeader
import com.gemwallet.android.testkit.mockGemValueHeader
import com.wallet.core.primitives.Asset
import io.mockk.every
import kotlinx.coroutines.runBlocking
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmSection
import uniffi.gemstone.GemConfirmStage
import uniffi.gemstone.GemConfirmTitle
import uniffi.gemstone.GemConfirmViewState
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemKeystoreAuthentication
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemTransactionHeader

internal fun GemConfirmation.stubViewState(): GemConfirmation = apply {
    every { viewState(any()) } answers {
        val screen = firstArg<GemConfirmScreen>()
        GemConfirmViewState(
            screen.button(),
            mockGemConfirmFeeRow(value = screen.feeValue(runBlocking { state() })),
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

internal fun symbolHeader(asset: Asset): GemConfirmHeader = mockGemConfirmHeader(header = GemTransactionHeader.Amount(mockGemValueHeader(title = GemLocalizedText.Text(asset.symbol))))
