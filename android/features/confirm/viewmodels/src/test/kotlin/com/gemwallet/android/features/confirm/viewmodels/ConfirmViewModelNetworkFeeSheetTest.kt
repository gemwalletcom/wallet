package com.gemwallet.android.features.confirm.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.confirm.pack
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.confirm.viewmodels.models.AcquireAssetRequest
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemConfirmFee
import com.gemwallet.android.testkit.mockGemConfirmLoad
import com.gemwallet.android.testkit.mockGemConfirmLoadOptions
import com.gemwallet.android.testkit.mockGemConfirmScreen
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.job
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAcquireAsset
import uniffi.gemstone.GemAcquireAssetFlow
import uniffi.gemstone.GemConfirmException
import uniffi.gemstone.GemConfirmHeader
import uniffi.gemstone.GemConfirmLoad
import uniffi.gemstone.GemConfirmPhase
import uniffi.gemstone.GemConfirmTransferService
import uniffi.gemstone.GemConfirmation
import uniffi.gemstone.GemInfoAction
import uniffi.gemstone.GemSwapPairSelection
import uniffi.gemstone.GemTransactionHeader
import uniffi.gemstone.GemTransferAmountResult
import uniffi.gemstone.confirmErrorInfo
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class ConfirmViewModelNetworkFeeSheetTest {
    private val testDispatcher = UnconfinedTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)
    private val payAsset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)
    private val account = mockAccount(chain = Chain.Solana)
    private val confirmService = mockk<GemConfirmTransferService>(relaxed = true)
    private val confirmation = mockk<GemConfirmation> {
        every { rowContents(any()) } returns emptyList()
        every { feeRateRows() } returns null
    }.stubViewState()
    private var model: ConfirmViewModel? = null

    @Before
    fun setUp() = Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = runTest(testDispatcher) {
        model?.viewModelScope?.coroutineContext?.job?.cancelAndJoin()
        Dispatchers.resetMain()
    }

    @Test
    fun networkFeeSheetShowsOncePerErrorAndStaysDismissed() = runTest(testDispatcher) {
        val viewModel = viewModel().also { model = it }
        advanceUntilIdle()

        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.value.phase)
        assertTrue(viewModel.isErrorSheetVisible.value)

        viewModel.dismissErrorSheet()
        advanceUntilIdle()

        assertEquals(GemConfirmPhase.FAILED, viewModel.screen.value.phase)
        assertFalse(viewModel.isErrorSheetVisible.value)

        viewModel.send(FinishConfirmAction { _, _ -> })
        advanceUntilIdle()

        assertTrue(viewModel.isErrorSheetVisible.value)
    }

    @Test
    fun refreshThatFindsTheSameProblemKeepsTheSheetAsTheUserLeftIt() = runTest(testDispatcher) {
        val problem = GemConfirmException.InsufficientNetworkFee(asset = asset.toGem(), requirement = null)
        val viewModel = viewModel { mockGemConfirmLoad(asset, fee = mockGemConfirmFee(GemTransferAmountResult.Error(problem))) }.also { model = it }
        advanceUntilIdle()

        assertEquals(GemConfirmPhase.READY, viewModel.screen.value.phase)
        assertTrue(viewModel.isErrorSheetVisible.value)

        viewModel.fetch()
        advanceUntilIdle()

        assertTrue(viewModel.isErrorSheetVisible.value)

        viewModel.dismissErrorSheet()
        viewModel.fetch()
        advanceUntilIdle()

        assertEquals(GemConfirmPhase.READY, viewModel.screen.value.phase)
        assertFalse(viewModel.isErrorSheetVisible.value)
    }

    @Test
    fun loadErrorCarriesItsTextAndInfoSheet() = runTest(testDispatcher) {
        val viewModel = viewModel().also { model = it }
        advanceUntilIdle()

        val error = requireNotNull(viewModel.loadError.value)
        assertEquals("Error", error.text)
        assertTrue(error.info?.sheet?.action is GemInfoAction.Acquire)

        val acquire = GemAcquireAsset(GemAcquireAssetFlow.FIAT, 10, GemSwapPairSelection(payAssetId = payAsset.id.toIdentifier(), receiveAssetId = asset.id.toIdentifier()))
        viewModel.acquire(asset, acquire)
        assertEquals(AcquireAssetRequest(asset = asset, acquire = acquire), viewModel.acquireRequest.value)
        assertEquals(payAsset.id, viewModel.acquireRequest.value?.swapPayAssetId)
        viewModel.dismissAcquire()
        assertEquals(null, viewModel.acquireRequest.value)
    }

    private fun viewModel(load: () -> GemConfirmLoad = { throw GemConfirmException.InsufficientNetworkFee(asset = asset.toGem(), requirement = null) }): ConfirmViewModel {
        val transfer = mockGemTransferData(asset = asset, value = BigInteger.TEN)
        every { confirmation.getCurrency() } returns Currency.USD.toGem()
        every { confirmation.errorInfo(any()) } answers { confirmErrorInfo(firstArg(), emptyList(), Currency.USD.toGem(), asset.id.toIdentifier(), asset.id.toIdentifier()) }
        every { confirmService.confirmation(any(), transfer, any()) } returns confirmation
        every { confirmation.screen() } returns mockGemConfirmScreen()
        every { confirmation.loadOptions() } returns mockGemConfirmLoadOptions()
        every { confirmation.header(any()) } returns GemConfirmHeader.Transaction(GemTransactionHeader.Symbol(asset.toGem()))
        coEvery { confirmation.state() } returns mockGemConfirmLoad(asset)
        coEvery { confirmation.load(any()) } answers { load() }
        return ConfirmViewModel(
            getSession = mockk<GetSession> {
                every { this@mockk() } returns MutableStateFlow(
                    mockSession(wallet = mockWallet(accounts = listOf(account))),
                )
            },
            confirmService = confirmService,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.Params.key to requireNotNull(transfer.pack()))),
            connectionStatusObserver = mockk(relaxed = true),
            ioDispatcher = testDispatcher,
            context = mockk<Context> {
                every { getString(any()) } returns "Error"
                every { getString(any(), *anyVararg()) } returns "Error"
            },
        )
    }
}
