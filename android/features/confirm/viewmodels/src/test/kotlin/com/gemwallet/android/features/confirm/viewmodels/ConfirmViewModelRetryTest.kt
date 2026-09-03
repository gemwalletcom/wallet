package com.gemwallet.android.features.confirm.viewmodels

import uniffi.gemstone.GemTransferAmount
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.GemTransferService
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.ConfirmState
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.domains.perpetual.toGem
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmPreload
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadMetadata
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmSimulationState
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransactionInputType
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
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
import com.wallet.core.primitives.PerpetualType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
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
import org.junit.Assert.assertEquals
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
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun retryAfterPreloadFailureRunsThePreloaderAgain() = runTest(testDispatcher) {
        val transfer = GemTransferData(
            inputType = GemTransactionInputType.Perpetual(asset.toGem(), PerpetualType.Open(mockPerpetualConfirmData(direction = PerpetualDirection.Long)).toGem()),
            recipient = GemRecipient(address = ""),
            value = BigInteger.TEN.toString(),
        )
        val viewModel = viewModel(transfer)
        runCurrent()
        coVerify(timeout = 5_000, exactly = 1) { confirmService.load(any(), any(), any()) }

        assertTrue(viewModel.state.first { it is ConfirmState.Error } is ConfirmState.Error)

        viewModel.send(FinishConfirmAction { _ -> })
        runCurrent()

        coVerify(timeout = 5_000, exactly = 2) { confirmService.load(any(), any(), any()) }
        assertTrue(viewModel.state.first { it is ConfirmState.Ready } is ConfirmState.Ready)
        assertEquals(asset, viewModel.feeAsset.first { it != null }?.asset)
    }

    private fun viewModel(transfer: GemTransferData): ConfirmViewModel {
        val input = GemConfirmInput(from = account.toGem(), transfer = transfer)
        every { confirmService.currency() } returns Currency.USD.toGem()
        every { confirmService.confirmInput(transfer) } returns input
        var calls = 0
        coEvery { confirmService.load(any(), any(), any()) } answers {
            calls += 1
            if (calls == 1) {
                throw IllegalStateException("preload failed")
            } else {
                GemConfirmLoad(
                    feeAssets = emptyList(),
                    simulation = GemConfirmSimulationState(simulation = null, addressNames = emptyList()),
                    preload = GemConfirmPreload(
                    confirmData = GemConfirmData(
                        fee = GemTransactionLoadFee(
                            fee = "1",
                            gasPriceType = GemGasPriceType.Regular(gasPrice = "1"),
                            gasLimit = "1",
                            options = GemFeeOptions(emptyMap()),
                            feeAsset = asset.id.chain.string,
                        ),
                        selectedPriority = FeePriority.Normal.toGem(),
                        feeRates = emptyList(),
                        metadata = GemTransactionLoadMetadata.None,
                        simulation = null,
                        input = input,
                    ),
                    metadata = mockGemConfirmMetadata(asset),
                    feeAsset = asset.toGem(),
                    amount = GemTransferAmountResult.Amount(GemTransferAmount(value = "1", networkFee = "1", isMaxAmount = false)),
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
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transferService.pack(transfer)))),
            transferService = uniffi.gemstone.GemTransferService(),
        )
    }
}
