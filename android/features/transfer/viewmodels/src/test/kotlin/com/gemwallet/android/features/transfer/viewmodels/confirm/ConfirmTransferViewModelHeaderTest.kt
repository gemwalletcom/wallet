package com.gemwallet.android.features.transfer.viewmodels.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockGemAssetBalance
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmMetadata
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import com.gemwallet.android.testkit.mockGemTransferData
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemConfirmFeeValue
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.TransactionInputType
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmTransferViewModelHeaderTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset()
    private val account = mockAccount(chain = Chain.Bitcoin)

    @get:Rule
    val confirm = ConfirmTransferRule(testDispatcher, account)

    @Test
    fun theHeadShowsWhatCoreComposedWhileTheFeeIsStillLoading() = runTest(testDispatcher) {
        val viewModel = viewModel(mockGemTransferData(inputType = TransactionInputType.Transfer(mockAsset().toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.valueOf(150_000)))

        val header = viewModel.header.first { it != null }

        assertEquals(symbolHeader(asset), header)
        assertEquals(GemConfirmFeeValue.Loading, viewModel.viewState.first { it != null }?.feeRow?.value)
        assertEquals(GemConfirmPhase.LOADING, viewModel.screen.value.phase)
    }

    private fun viewModel(transfer: GemTransferData): ConfirmTransferViewModel {
        val confirmation = mockk<GemConfirmation> {
            every { rowContents(any()) } returns emptyList()
            every { networkFeeScreen(any()) } returns null
        }.stubViewState()
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header(any()) } returns symbolHeader(asset)
        coEvery { confirmation.state() } returns
            mockGemConfirmLoad(
                transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE),
                sender = mockAccount(chain = asset.id.chain).toGem(),
                feeAsset = asset.toGem(),
                metadata = mockGemConfirmMetadata(assetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true), feeAssetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true)),
                simulation = mockGemConfirmSimulationState(chain = asset.id.chain.string),
            ).copy(transfer = transfer)
        coEvery { confirmation.load(any()) } coAnswers { awaitCancellation() }
        every { confirm.confirmService.confirmation(any(), transfer, any()) } returns confirmation
        return confirm.viewModel(transfer)
    }
}
