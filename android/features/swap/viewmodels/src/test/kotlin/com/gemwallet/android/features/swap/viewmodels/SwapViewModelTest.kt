package com.gemwallet.android.features.swap.viewmodels

import android.content.Context
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.swap.cases.RequestSwapQuotes
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBalance
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPrice
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockBalance
import com.gemwallet.android.testkit.mockGemSwapSession
import com.gemwallet.android.testkit.mockGemSwapTransfer
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockSwapQuoteRequestParams
import com.gemwallet.android.testkit.mockSwapQuotesResult
import com.gemwallet.android.testkit.mockSwapperQuote
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.infoSheet
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.clearMocks
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.job
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemSwapPairSelection
import uniffi.gemstone.GemSwapPairSuggestion
import uniffi.gemstone.GemSwapQuoteServiceInterface
import uniffi.gemstone.GemSwapRequest
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.SwapProvider
import uniffi.gemstone.SwapperException
import uniffi.gemstone.SwapperQuote
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class SwapViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    private val solAsset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)
    private val usdcAsset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)
    private val solInfo = mockAssetInfo(
        owner = mockAccount(chain = Chain.Solana),
        asset = solAsset,
        balance = mockAssetBalance(asset = solAsset, balance = mockBalance(available = BigInteger("1000000000"))),
        walletId = mockWalletId(),
    )
    private val usdcInfo = mockAssetInfo(owner = mockAccount(chain = Chain.Solana), asset = usdcAsset, walletId = mockWalletId())

    private val getSession = mockk<GetSession>(relaxed = true) {
        every { this@mockk() } returns MutableStateFlow(null)
    }
    private val getAssetInfo = mockk<GetAssetInfo>(relaxed = true) {
        every { this@mockk(solAsset.id) } returns flowOf(solInfo)
        every { this@mockk(usdcAsset.id) } returns flowOf(usdcInfo)
    }
    private val requestSwapQuotes = mockk<RequestSwapQuotes>(relaxed = true)
    private val swapQuoteService = mockk<GemSwapQuoteServiceInterface>(relaxed = true) {
        every { slippageBps() } returns null
        coEvery { suggestPair(any()) } returns null
        every { newSession() } answers { mockGemSwapSession() }
        every { getCurrency() } returns Currency.USD.toGem()
        every { selectPairAsset(any(), any(), any()) } answers { pairSelection }
    }

    private var pairSelection = GemSwapPairSelection(payAssetId = null, receiveAssetId = null)

    private val createdViewModels = mutableListOf<SwapViewModel>()

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        clearMocks(getSession, getAssetInfo, requestSwapQuotes)
        clearMocks(swapQuoteService, answers = false)
        every { getSession() } returns MutableStateFlow(null)
        every { getAssetInfo(solAsset.id) } returns flowOf(solInfo)
        every { getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns emptyFlow()
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        createdViewModels.forEach { it.viewModelScope.coroutineContext.job.cancelAndJoin() }
        createdViewModels.clear()
        Dispatchers.resetMain()
    }

    private fun createViewModel(savedStateHandle: SavedStateHandle) = SwapViewModel(
        getAssetInfo = getAssetInfo,
        requestSwapQuotes = requestSwapQuotes,
        swapQuoteService = swapQuoteService,
        savedStateHandle = savedStateHandle,
        ioDispatcher = testDispatcher,
        context = mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },

    ).also { createdViewModels += it }

    private fun swapSavedState(from: String = solAsset.id.toIdentifier(), to: String? = usdcAsset.id.toIdentifier()) = SavedStateHandle(
        mapOf(
            RouteArgument.FromAssetId.key to from,
            RouteArgument.ToAssetId.key to to,
        ),
    )

    @Test
    fun `the pair is refreshed once, for both legs together`() = runTest(testDispatcher) {
        val wallet = mockWallet(
            accounts = listOf(mockAccount(chain = solAsset.id.chain), mockAccount(chain = usdcAsset.id.chain)),
        )
        every { getSession() } returns MutableStateFlow(mockSession(wallet = wallet))

        createViewModel(swapSavedState())
        advanceUntilIdle()

        coVerify(exactly = 1) { swapQuoteService.refreshPair(listOf(solAsset.id.toIdentifier(), usdcAsset.id.toIdentifier())) }
    }

    @Test
    fun `init keeps an already selected pair and asks for no suggestion`() = runTest(testDispatcher) {
        val savedState = swapSavedState()

        createViewModel(savedState)
        advanceUntilIdle()

        coVerify(exactly = 0) { swapQuoteService.suggestPair(any()) }
        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
    }

    @Test
    fun `init applies the suggested pair when the screen opens empty`() = runTest(testDispatcher) {
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(mockSession(wallet = wallet))
        coEvery { swapQuoteService.suggestPair(null) } returns GemSwapPairSuggestion(
            payAssetId = solAsset.id.toIdentifier(),
            receiveAssetId = usdcAsset.id.toIdentifier(),
        )
        val savedState = SavedStateHandle()

        createViewModel(savedState)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
    }

    @Test
    fun `a typed slippage is saved when the sheet closes`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        viewModel.openSlippage()
        viewModel.onSlippageAuto(false)
        viewModel.onSlippageInput("2")
        viewModel.closeSlippage()
        advanceUntilIdle()

        coVerify { swapQuoteService.setSlippageBps(200u) }
        assertEquals(200u, viewModel.selectedSlippage.value)
        assertNull(viewModel.slippage.value)
    }

    @Test
    fun `a slippage outside the allowed range is not saved`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        viewModel.openSlippage()
        viewModel.onSlippageAuto(false)
        viewModel.onSlippageInput("25")
        viewModel.closeSlippage()
        advanceUntilIdle()

        coVerify(exactly = 0) { swapQuoteService.setSlippageBps(any()) }
        assertNull(viewModel.selectedSlippage.value)
    }

    @Test
    fun `onSelect updates pay asset from empty state`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        pairSelection = GemSwapPairSelection(solAsset.id.toIdentifier(), null)
        viewModel.onSelect(SwapItemType.Pay, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertNull(savedState.get<String?>(RouteArgument.ToAssetId.key))
        assertEquals(solAsset.id, viewModel.payAsset.value?.id())
    }

    @Test
    fun `onSelect keeps opposite asset when pair differs`() = runTest(testDispatcher) {
        val savedState = swapSavedState()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        pairSelection = GemSwapPairSelection(solAsset.id.toIdentifier(), usdcAsset.id.toIdentifier())
        viewModel.onSelect(SwapItemType.Receive, usdcAsset.id)
        advanceUntilIdle()

        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
    }

    @Test
    fun `selecting receive asset preserves pay amount`() = runTest(testDispatcher) {
        val savedState = swapSavedState(to = null)

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.payValue.setTextAndPlaceCursorAtEnd("1.5")
        Snapshot.sendApplyNotifications()
        pairSelection = GemSwapPairSelection(solAsset.id.toIdentifier(), usdcAsset.id.toIdentifier())
        viewModel.onSelect(SwapItemType.Receive, usdcAsset.id)
        advanceUntilIdle()

        assertEquals("1.5", viewModel.payValue.text.toString())
        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
    }

    @Test
    fun `receiving the asset being paid with turns the pair around and clears the amount`() = runTest(testDispatcher) {
        val savedState = swapSavedState(to = usdcAsset.id.toIdentifier())

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        pairSelection = GemSwapPairSelection(usdcAsset.id.toIdentifier(), solAsset.id.toIdentifier())
        viewModel.onSelect(SwapItemType.Receive, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertEquals("the amount was typed in the asset that moved to the other side", "", viewModel.payValue.text.toString())
    }

    @Test
    fun `paying with the asset being received turns the pair around`() = runTest(testDispatcher) {
        val savedState = swapSavedState(
            from = usdcAsset.id.toIdentifier(),
            to = solAsset.id.toIdentifier(),
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        pairSelection = GemSwapPairSelection(solAsset.id.toIdentifier(), usdcAsset.id.toIdentifier())
        viewModel.onSelect(SwapItemType.Pay, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
    }

    @Test
    fun `quote refresh does not replace swapping state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )

        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val savedState = swapSavedState()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        val quotesState = seedReadyQuote(viewModel, quotesFlow)
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)

        var confirmCalls = 0
        viewModel.swap { confirmCalls++ }
        awaitCondition { viewModel.uiState.value.isTransferLoading }

        quotesFlow.emit(quotesState.copy(items = listOf(mockSwapperQuote(toValue = BigInteger("2600000")))))
        advanceUntilIdle()

        assertTrue(viewModel.uiState.value.isTransferLoading)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)
        assertEquals(0, confirmCalls)

        confirmInputGate.complete(Unit)
        awaitCondition { confirmCalls == 1 }
    }

    @Test
    fun `transfer data error keeps quote visible and routes retry through transfer state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.NoQuoteAvailable()

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.errorText != null }

        assertEquals(GemInfoTopic.NoQuote.infoSheet(), viewModel.uiState.value.errorInfo)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)
    }

    @Test
    fun `a transfer failure Core cannot retry leaves the button tappable and retries the transfer`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.InvalidRoute()

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.errorText != null }

        val state = viewModel.uiState.value
        assertEquals(R.string.wallet_swap, state.actionTitle)
        assertEquals(ButtonState.Enabled, state.buttonState)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)

        coEvery { swapQuoteService.getTransfer(any()) } returns mockGemSwapTransfer(from = solInfo.owner!!, toAddress = "0xconfirm")

        var confirmed: GemTransferData? = null
        viewModel.onPrimaryAction(
            onConfirm = { confirmed = it.data },
            onShowPriceImpactWarning = {},
            authorize = { it() },
        )
        awaitCondition { confirmed != null }

        assertEquals(solInfo.owner!!.address, confirmed?.recipient?.address)
    }

    @Test
    fun `quote changing actions clear transfer error state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.NoQuoteAvailable()

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)
        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.errorText != null }

        viewModel.setProvider(SwapProvider.UNISWAP_V3)
        advanceUntilIdle()

        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.errorText != null }

        viewModel.payValue.setTextAndPlaceCursorAtEnd("2")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.isQuoteLoading }
    }

    @Test
    fun `quote refresh stays paused after confirm handoff until screen restarts`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        val refreshEnabledFlow = slot<Flow<Boolean>>()
        every {
            requestSwapQuotes.invoke(any(), any(), capture(refreshEnabledFlow), any(), any(), any())
        } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )

        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        val refreshStates = mutableListOf<Boolean>()
        val collectJob = launch {
            refreshEnabledFlow.captured.toList(refreshStates)
        }

        seedReadyQuote(viewModel, quotesFlow)

        viewModel.setRefreshEnabled(true)
        advanceUntilIdle()
        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.isTransferLoading }
        confirmInputGate.complete(Unit)
        awaitCondition { viewModel.uiState.value.buttonState == ButtonState.Enabled && viewModel.uiState.value.errorText == null }
        advanceUntilIdle()
        assertEquals(false, refreshStates.last())

        viewModel.setRefreshEnabled(false)
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        awaitCondition { refreshStates.size >= 6 && refreshStates.last() }

        collectJob.cancel()
        assertEquals(listOf(false, true, false), refreshStates.take(3))
        assertEquals(false, refreshStates[3])
        assertEquals(true, refreshStates.last())
        assertEquals(2, refreshStates.count { it })
    }

    @Test
    fun `quote fetch started callback shows quote loading for refreshes`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        val onFetchStarted = slot<(GemSwapRequest) -> Unit>()
        every {
            requestSwapQuotes.invoke(any(), any(), any(), capture(onFetchStarted), any(), any())
        } returns quotesFlow

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        val seededQuotes = seedReadyQuote(viewModel, quotesFlow)
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)

        onFetchStarted.captured(seededQuotes.requestKey)
        advanceUntilIdle()

        assertTrue(viewModel.uiState.value.isQuoteLoading)
    }

    @Test
    fun `confirm callback runs before transfer loading clears`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        var wasTransferLoadingOnConfirm = false
        viewModel.swap {
            wasTransferLoadingOnConfirm = viewModel.uiState.value.isTransferLoading
        }
        awaitCondition { viewModel.uiState.value.isTransferLoading }
        confirmInputGate.complete(Unit)
        awaitCondition { !viewModel.uiState.value.isTransferLoading }

        assertTrue(wasTransferLoadingOnConfirm)
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)
    }

    @Test
    fun `confirm params keep frozen from amount while transfer is in flight`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        var confirmInput: GemTransferData? = null
        viewModel.swap { input ->
            confirmInput = input.data
        }
        awaitCondition { viewModel.uiState.value.isTransferLoading }

        viewModel.payValue.setTextAndPlaceCursorAtEnd("2")
        confirmInputGate.complete(Unit)
        awaitCondition { confirmInput != null }

        assertEquals(BigInteger("1000000000"), confirmInput?.value)
    }

    @Test
    fun `onPrimaryAction does not build swap params until authorize runs`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(mockSession(wallet = wallet))

        var swapCalls = 0
        stubBuildConfirmInput { swapCalls += 1 }

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        seedReadyQuote(viewModel, quotesFlow)

        var authorized: (() -> Unit)? = null
        viewModel.onPrimaryAction(
            onConfirm = {},
            onShowPriceImpactWarning = {},
            authorize = { action -> authorized = action },
        )
        advanceUntilIdle()

        assertNotNull("the swap must be handed to authorize, not run directly", authorized)
        assertEquals(0, swapCalls)
    }

    @Test
    fun `onPrimaryAction shows price impact warning before swap`() = runTest(testDispatcher) {
        every { getAssetInfo(solAsset.id) } returns flowOf(solInfo.copy(price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 100.0))))
        every { getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo.copy(price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 1.0))))

        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )

        var swapCalls = 0
        stubBuildConfirmInput { swapCalls += 1 }

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        var showWarningCalls = 0
        var confirmCalls = 0
        viewModel.onPrimaryAction(
            onConfirm = { confirmCalls += 1 },
            onShowPriceImpactWarning = { showWarningCalls += 1 },
            authorize = { it() },
        )
        advanceUntilIdle()

        assertEquals(1, showWarningCalls)
        assertEquals(0, swapCalls)
        assertEquals(0, confirmCalls)
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)
    }

    @Test
    fun `taking the offered minimum fills the pay field with it`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, quotesFlow, SwapperException.InputAmountException("500000000"))
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.swap_use_minimum_amount }

        viewModel.onPrimaryAction(onConfirm = {}, onShowPriceImpactWarning = {}, authorize = { it() })
        advanceUntilIdle()

        assertEquals("0.5", viewModel.payValue.text.toString())
    }

    @Test
    fun `small minimum amount uses editable decimal notation`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, quotesFlow, SwapperException.InputAmountException("1"))
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.swap_use_minimum_amount }

        viewModel.onPrimaryAction(onConfirm = {}, onShowPriceImpactWarning = {}, authorize = { it() })
        advanceUntilIdle()

        assertEquals("0.000000001", viewModel.payValue.text.toString())
    }

    private suspend fun failQuote(viewModel: SwapViewModel, quotesFlow: MutableSharedFlow<SwapQuotesResult?>, error: SwapperException) {
        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        testDispatcher.scheduler.advanceUntilIdle()
        quotesFlow.emit(mockSwapQuotesResult(params = mockSwapQuoteRequestParams(pay = solInfo, receive = usdcInfo), err = error))
        testDispatcher.scheduler.advanceUntilIdle()
    }

    private fun stubBuildConfirmInput(beforeReturn: suspend () -> Unit = {}) {
        coEvery { swapQuoteService.getTransfer(any()) } coAnswers {
            beforeReturn()
            val quote = firstArg<SwapperQuote>()
            mockGemSwapTransfer(
                from = solInfo.owner!!,
                fromAmount = quote.fromValue,
                toAmount = quote.toValue,
                useMaxAmount = quote.request.options.useMaxAmount,
                toAddress = "0xconfirm",
            )
        }
    }

    private fun awaitCondition(timeoutMs: Long = 2_000, condition: () -> Boolean) {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (!condition() && System.currentTimeMillis() < deadline) {
            testDispatcher.scheduler.advanceUntilIdle()
            Thread.sleep(10)
        }
        assertTrue("condition not met within ${timeoutMs}ms", condition())
    }

    private suspend fun seedReadyQuote(viewModel: SwapViewModel, quotesFlow: MutableSharedFlow<SwapQuotesResult?>, quote: SwapperQuote = mockSwapperQuote()): SwapQuotesResult {
        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.isQuoteLoading }

        val quotesState = mockSwapQuotesResult(params = mockSwapQuoteRequestParams(pay = solInfo, receive = usdcInfo), items = listOf(quote))
        quotesFlow.emit(quotesState)
        testDispatcher.scheduler.advanceUntilIdle()
        awaitCondition { viewModel.uiState.value.buttonState == ButtonState.Enabled && viewModel.uiState.value.errorText == null }
        return quotesState
    }
}
