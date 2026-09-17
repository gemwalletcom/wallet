package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import uniffi.gemstone.GemTransferAmount
import uniffi.gemstone.GemTransferAmountResult
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmPreload
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GasPriceType
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualType
import uniffi.gemstone.TransactionInputType
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockPerpetualConfirmData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.PerpetualDirection
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.job
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelRetryTest {


    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAssetHyperCoreUBTC()
    private val account = mockAccount(chain = Chain.HyperCore)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmation = mockk<GemConfirmation>()
    private var model: ConfirmViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun retryAfterPreloadFailureRunsThePreloaderAgain() = runTest(testDispatcher) {
        val transfer = mockGemTransferData(
            inputType = TransactionInputType.Perpetual(asset.toGem(), PerpetualType.Open(mockPerpetualConfirmData(direction = PerpetualDirection.Long))),
            value = BigInteger.TEN,
        )
        val viewModel = viewModel(transfer).also { model = it }
        advanceUntilIdle()

        coVerify(exactly = 1) { confirmation.load(any()) }
        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.value.phase)

        viewModel.send(FinishConfirmAction { _ -> })
        advanceUntilIdle()

        coVerify(exactly = 2) { confirmation.load(any()) }
        assertEquals(GemConfirmPhase.READY, viewModel.screen.value.phase)
        assertEquals(asset, viewModel.feeAsset.value?.asset)
    }

    private fun viewModel(transfer: GemTransferData): ConfirmViewModel {
        val input = GemConfirmInput(from = account.toGem(), transfer = transfer)
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.insufficientNetworkFeeBuyAmount() } returns 10
        every { confirmService.confirmation(any(), transfer, any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        coEvery { confirmation.state() } returns mockGemConfirmLoad(asset)
        var calls = 0
        coEvery { confirmation.load(any()) } answers {
            calls += 1
            if (calls == 1) {
                throw IllegalStateException("preload failed")
            } else {
                mockGemConfirmLoad(
                    asset = asset,
                    preload = GemConfirmPreload(
                        confirmData = GemConfirmData(
                            fee = GemTransactionLoadFee(
                                fee = BigInteger.ONE,
                                gasPriceType = GasPriceType.Regular(gasPrice = BigInteger.ONE),
                                gasLimit = BigInteger.ONE,
                                options = GemFeeOptions(emptyMap()),
                                feeAsset = asset.id.chain.string,
                            ),
                            additionalFees = emptyList(),
                            selectedPriority = FeePriority.Normal.toGem(),
                            feeRates = emptyList(),
                            metadata = GemTransactionLoadMetadata.None,
                            simulation = null,
                            input = input,
                        ),
                        amount = GemTransferAmountResult.Amount(GemTransferAmount(value = BigInteger.ONE, networkFee = BigInteger.ONE, isMaxAmount = false)),
                    ),
                )
            }
        }
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            buildConfirmProperties = mockk(relaxed = true),
            confirmService = confirmService,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack()))),
            ioDispatcher = testDispatcher,
            context = mockk<Context> { every { getString(any()) } returns "Error"; every { getString(any(), *anyVararg()) } returns "Error" },
        )
    }
}
