package com.gemwallet.android.features.confirm.viewmodels

import uniffi.gemstone.GemTransferAmount
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemTransferService
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.confirm.perpetual
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockPerpetualConfirmData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelRetryTest {

    private val transferService = GemTransferService()

    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAssetHyperCoreUBTC()
    private val account = mockAccount(chain = Chain.HyperCore)
    private val preloader = mockk<SignerPreloaderProxy>()

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun retryAfterPreloadFailureRunsThePreloaderAgain() = runTest(testDispatcher) {
        val input = GemTransferData.perpetual(
            asset = asset,
            perpetualType = PerpetualType.Open(mockPerpetualConfirmData(direction = PerpetualDirection.Long)),
            value = BigInteger.TEN,
        ).confirmInput(account)
        val viewModel = viewModel(input)
        runCurrent()
        coVerify(timeout = 5_000, exactly = 1) { preloader.preload(any(), any(), any(), any()) }

        assertTrue(viewModel.state.first { it is ConfirmState.Error } is ConfirmState.Error)

        viewModel.send(FinishConfirmAction { _ -> })
        runCurrent()

        coVerify(timeout = 5_000, exactly = 2) { preloader.preload(any(), any(), any(), any()) }
    }

    private fun viewModel(input: GemConfirmInput): ConfirmViewModel {
        var calls = 0
        coEvery { preloader.preload(any(), any(), any(), any()) } answers {
            calls += 1
            if (calls == 1) {
                throw IllegalStateException("preload failed")
            } else {
                SignerPreloaderProxy.Preload(
                    signerParams = mockk(relaxed = true),
                    simulation = null,
                    amount = GemTransferAmountResult.Amount(GemTransferAmount(value = "1", networkFee = "1", isMaxAmount = false)),
                    feeAsset = asset,
                )
            }
        }
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                io.mockk.every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            getWalletAssets = mockk(relaxed = true),
            getAssetInfo = mockk(relaxed = true),
            syncMissingAssets = mockk(relaxed = true),
            confirmLoader = ConfirmLoader(preloader),
            getFeeAssets = mockk(relaxed = true),
            confirmTransaction = mockk(relaxed = true),
            buildConfirmProperties = mockk(relaxed = true),
            explorerService = mockk(relaxed = true),
            getAddressName = mockk(relaxed = true),
            getAddressNames = mockk(relaxed = true),
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transferService.pack(input)))),
            feeService = uniffi.gemstone.GemFeeService(),
            transferService = uniffi.gemstone.GemTransferService(),
            simulationFormatter = mockk(relaxed = true),
            perpetualService = mockk(relaxed = true),
            swapQuoteService = mockk(relaxed = true),
        )
    }
}
