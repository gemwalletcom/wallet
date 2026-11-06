package com.gemwallet.android.features.transfer.viewmodels.confirm

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemAssetBalance
import com.gemwallet.android.testkit.mockGemConfirmFee
import com.gemwallet.android.testkit.mockGemConfirmHeader
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmMetadata
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemConfirmSimulationState
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockHyperCoreBtc
import com.gemwallet.android.testkit.mockHyperCoreUsdc
import com.gemwallet.android.testkit.mockPerpetualConfirmData
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemConfirmAction
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemPerpetualConfirmedAction
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemSubmitMessage
import uniffi.gemstone.GemSubmitResult
import uniffi.gemstone.GemTransferAmount
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualType
import uniffi.gemstone.TransactionInputType
import uniffi.gemstone.feeAmount
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmTransferViewModelRetryTest {

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockHyperCoreBtc()
    private val account = mockAccount(chain = Chain.HyperCore)
    private val confirmation = mockk<GemConfirmation> {
        every { rowContents(any()) } returns emptyList()
        every { networkFeeScreen(any()) } returns null
    }.stubViewState()

    private val perpetualTransfer = mockGemTransferData(
        inputType = TransactionInputType.Perpetual(
            asset.toGem(),
            PerpetualType.Open(
                mockPerpetualConfirmData(
                    direction = PerpetualDirection.Long.toGem(), marginType = PerpetualMarginType.Cross.toGem(),
                    baseAsset = mockHyperCoreUsdc().toGem(),
                    price = "100.0", fiatValue = 100.0, size = "1.0", slippage = 2.0, leverage = 1u, marketPrice = 100.0, marginAmount = 100.0,
                ),
            ),
        ),
        recipient = GemRecipient(address = "recipient"),
        value = BigInteger.TEN,
    )

    @get:Rule
    val confirm = ConfirmTransferRule(testDispatcher, account)

    @Test
    fun retryAndRefreshUseTheCurrentFeeSelection() = runTest(testDispatcher) {
        val viewModel = viewModel(perpetualTransfer)
        advanceUntilIdle()

        coVerify(exactly = 1) { confirmation.load(any()) }
        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.value.phase)

        viewModel.load()
        advanceUntilIdle()

        coVerify(exactly = 2) { confirmation.load(any()) }
        assertEquals(GemConfirmPhase.READY, viewModel.screen.value.phase)

        viewModel.changeFeePriority(FeePriority.Fast)
        advanceUntilIdle()
        viewModel.load()
        advanceUntilIdle()
        coVerify(exactly = 4) { confirmation.load(any()) }
        coVerify(exactly = 2) { confirmation.load(match { it.feeSelection == GemConfirmFeeSelection.Priority(FeePriority.Fast.toGem()) }) }
    }

    @Test
    fun aFailedSendShowsItsErrorAndConfirmSendsAgainWithoutReloading() = runTest(testDispatcher) {
        val viewModel = viewModel(mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE))
        coEvery { confirmation.submit() } throws GemConfirmException.Broadcast("rejected")
        viewModel.load()
        advanceUntilIdle()

        viewModel.startExecute()
        viewModel.submit()
        advanceUntilIdle()

        assertNotNull(viewModel.executeErrorText.value)
        assertEquals(GemConfirmPhase.READY, viewModel.screen.value.phase)
        assertEquals(GemConfirmAction.EXECUTE, viewModel.action())
        coVerify(exactly = 2) { confirmation.load(any()) }

        viewModel.dismissExecuteError()
        viewModel.startExecute()
        viewModel.submit()
        advanceUntilIdle()

        assertNotNull(viewModel.executeErrorText.value)
        coVerify(exactly = 2) { confirmation.submit() }
        coVerify(exactly = 2) { confirmation.load(any()) }
    }

    @Test
    fun aConfirmWaitingOnAuthStopsRefreshingAndCancelReturnsToALiveConfirm() = runTest(testDispatcher) {
        val viewModel = viewModel(mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE))
        viewModel.load()
        advanceUntilIdle()

        viewModel.startExecute()
        assertEquals(GemConfirmPhase.CONFIRMING, viewModel.screen.value.phase)
        assertFalse(viewModel.screen.value.refreshes())
        assertNull(viewModel.action())

        viewModel.cancelExecute()
        assertEquals(GemConfirmPhase.READY, viewModel.screen.value.phase)
        assertTrue(viewModel.screen.value.refreshes())
        assertEquals(GemConfirmAction.EXECUTE, viewModel.action())
        coVerify(exactly = 0) { confirmation.submit() }
    }

    @Test
    fun aConfirmedOrderShowsItsToastOnWhicheverScreenTheUserLandsOn() = runTest(testDispatcher) {
        val viewModel = viewModel(perpetualTransfer)
        coEvery { confirmation.submit() } returns GemSubmitResult.Sent(
            hashes = listOf("0xhash"),
            message = GemSubmitMessage.Confirmed(GemLocalizedText.PerpetualConfirmed(GemPerpetualConfirmedAction.Open(uniffi.gemstone.PerpetualDirection.LONG))),
        )
        viewModel.load()
        advanceUntilIdle()
        viewModel.startExecute()
        viewModel.submit()
        advanceUntilIdle()

        assertEquals("0xhash", viewModel.sentTransactionHash.value)
        val toast = confirm.toastPresenter.toastEvents.first()
        assertEquals(R.string.perpetual_open_direction.toString(), toast.title)
        assertEquals(R.drawable.ic_check_circle, toast.image)
    }

    private fun viewModel(transfer: GemTransferData): ConfirmTransferViewModel {
        every { confirmation.errorInfo(any()) } returns null
        every { confirm.confirmService.confirmation(any(), transfer, any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header(any()) } returns mockGemConfirmHeader()
        coEvery { confirmation.state() } returns
            mockGemConfirmLoad(
                transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE),
                sender = mockAccount(chain = asset.id.chain).toGem(),
                feeAsset = asset.toGem(),
                metadata = mockGemConfirmMetadata(assetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true), feeAssetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true)),
                simulation = mockGemConfirmSimulationState(chain = asset.id.chain.string),
            )
        var calls = 0
        coEvery { confirmation.load(any()) } answers {
            calls += 1
            if (calls == 1) {
                throw IllegalStateException("preload failed")
            } else {
                mockGemConfirmLoad(
                    transfer = mockGemTransferData(inputType = TransactionInputType.Transfer(asset.toGem()), recipient = GemRecipient(address = "recipient"), value = BigInteger.ONE),
                    sender = mockAccount(chain = asset.id.chain).toGem(),
                    feeAsset = asset.toGem(),
                    metadata = mockGemConfirmMetadata(assetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true), feeAssetBalance = mockGemAssetBalance(assetId = asset.id.toIdentifier(), isActive = true)),
                    simulation = mockGemConfirmSimulationState(chain = asset.id.chain.string),
                    fee = mockGemConfirmFee(
                        value = BigInteger.ONE,
                        formatted = feeAmount(mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18u).toGem(), BigInteger.ONE, null, Currency.USD.toGem()),
                        selectedPriority = FeePriority.Normal.toGem(),
                        amount = GemTransferAmountResult.Amount(GemTransferAmount(value = BigInteger.ONE, networkFee = BigInteger.ONE, isMaxAmount = false)),
                    ),
                )
            }
        }
        return confirm.viewModel(transfer)
    }
}
