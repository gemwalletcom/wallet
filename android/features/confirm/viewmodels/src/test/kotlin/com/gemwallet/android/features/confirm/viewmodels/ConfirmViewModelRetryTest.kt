package com.gemwallet.android.features.confirm.viewmodels

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
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GasPriceType
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmScreen
import uniffi.gemstone.GemConfirmSession
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PerpetualType
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.TransactionInputType
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmMetadata
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
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelRetryTest {


    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAssetHyperCoreUBTC()
    private val account = mockAccount(chain = Chain.HyperCore)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmSession = mockk<GemConfirmSession>()
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
        val transfer = GemTransferData(
            inputType = TransactionInputType.Perpetual(asset.toGem(), PerpetualType.Open(mockPerpetualConfirmData(direction = PerpetualDirection.Long))),
            recipient = GemRecipient(address = ""),
            value = BigInteger.TEN,
        )
        val viewModel = viewModel(transfer).also { model = it }
        runCurrent()
        coVerify(timeout = 5_000, exactly = 1) { confirmSession.load(any()) }

        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.first { it.phase == GemConfirmPhase.FAILED }.phase)

        viewModel.send(FinishConfirmAction { _ -> })
        runCurrent()

        coVerify(timeout = 5_000, exactly = 2) { confirmSession.load(any()) }
        assertEquals(GemConfirmPhase.READY, viewModel.screen.first { it.phase == GemConfirmPhase.READY }.phase)
        assertEquals(asset, viewModel.feeAsset.first { it != null }?.asset)
    }

    private fun viewModel(transfer: GemTransferData): ConfirmViewModel {
        val input = GemConfirmInput(from = account.toGem(), transfer = transfer)
        every { confirmSession.getCurrency() } returns Currency.USD.toGem()
        every { confirmSession.insufficientNetworkFeeBuyAmount() } returns 10
        every { confirmService.session(any(), transfer, any()) } returns confirmSession
        every { confirmSession.screen() } returns GemConfirmScreen(GemConfirmPhase.LOADING, false, false, null)
        coEvery { confirmSession.state() } returns mockGemConfirmLoad(asset, preload = null)
        var calls = 0
        coEvery { confirmSession.load(any()) } answers {
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
        )
    }
}
