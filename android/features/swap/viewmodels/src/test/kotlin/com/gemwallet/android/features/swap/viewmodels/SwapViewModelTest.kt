package com.gemwallet.android.features.swap.viewmodels

import com.gemwallet.android.ui.R
import android.content.Context
import com.gemwallet.android.ui.components.InfoSheetEntity
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.swap.cases.RequestSwapQuotes
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.swap.AssetRatePair
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.model.AssetBalance
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.testkit.mockGemSwapTransfer
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockGemSwapSession
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockSwapQuoteRequestParams
import com.gemwallet.android.testkit.mockSwapQuotesResult
import com.gemwallet.android.testkit.mockSwapperQuote
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapPriceImpactUIModel
import com.gemwallet.android.ui.models.swap.SwapProviderUIModel
import uniffi.gemstone.SwapPriceImpactType
import io.mockk.clearMocks
import io.mockk.coEvery
import uniffi.gemstone.GemSwapPairSelection
import uniffi.gemstone.GemSwapPairSuggestion
import uniffi.gemstone.GemSwapQuoteServiceInterface
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.slot
import io.mockk.unmockkObject
import java.math.BigInteger
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.cancel
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
import uniffi.gemstone.SwapProvider
import uniffi.gemstone.SwapperQuote
import uniffi.gemstone.SwapperException
import uniffi.gemstone.GemSwapRequest

@OptIn(ExperimentalCoroutinesApi::class)
class SwapViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    private val solAsset = mockAssetSolana()
    private val usdcAsset = mockAssetSolanaUSDC()
    private val solInfo = mockAssetInfo(
        asset = solAsset,
        balance = AssetBalance.create(solAsset, available = BigInteger("1000000000")),
    )
    private val usdcInfo = mockAssetInfo(asset = usdcAsset)

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
        every { selectPairAsset(any(), any(), any()) } answers { pairSelection }
    }

    private var pairSelection = GemSwapPairSelection(payAssetId = null, receiveAssetId = null)

    private val createdViewModels = mutableListOf<SwapViewModel>()

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        mockkObject(SwapDetailsUIModelFactory)
        clearMocks(getSession, getAssetInfo, requestSwapQuotes)
        clearMocks(swapQuoteService, answers = false)
        every { getSession() } returns MutableStateFlow(null)
        every { getAssetInfo(solAsset.id) } returns flowOf(solInfo)
        every { getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns emptyFlow()
        every { SwapDetailsUIModelFactory.create(any()) } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        createdViewModels.forEach { it.viewModelScope.coroutineContext.job.cancelAndJoin() }
        createdViewModels.clear()
        Dispatchers.resetMain()
        unmockkObject(SwapDetailsUIModelFactory)
    }

    private fun createViewModel(savedStateHandle: SavedStateHandle) = SwapViewModel(
        getAssetInfo = getAssetInfo,
        requestSwapQuotes = requestSwapQuotes,
        swapQuoteService = swapQuoteService,
        savedStateHandle = savedStateHandle,
        context = mockk<Context> {
            every { getString(any()) } returns "Error"
            every { getString(any(), *anyVararg()) } returns "Error"
        },

    ).also { createdViewModels += it }

    private fun swapSavedState(
        from: String = solAsset.id.toIdentifier(),
        to: String? = usdcAsset.id.toIdentifier(),
    ) = SavedStateHandle(
        mapOf(
            RouteArgument.FromAssetId.key to from,
            RouteArgument.ToAssetId.key to to,
        )
    )

    @Test
    fun `both legs of the pair are subscribed for live prices`() = runTest(testDispatcher) {
        val wallet = mockWallet(
            accounts = listOf(mockAccount(chain = solAsset.id.chain), mockAccount(chain = usdcAsset.id.chain)),
        )
        every { getSession() } returns MutableStateFlow(mockSession(wallet = wallet))

        createViewModel(swapSavedState())
        advanceUntilIdle()

        coVerify(exactly = 1) { swapQuoteService.addPrices(listOf(solAsset.id.toIdentifier())) }
        coVerify(exactly = 1) { swapQuoteService.addPrices(listOf(usdcAsset.id.toIdentifier())) }
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
    fun `setSlippage persists user preference`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        viewModel.setSlippage(200u)
        advanceUntilIdle()

        coVerify { swapQuoteService.setSlippageBps(200u) }
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
            mockSession(wallet = wallet)
        )

        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val savedState = swapSavedState()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        val quotesState = seedReadyQuote(viewModel, quotesFlow)
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)
        assertEquals(BigInteger("2500000"), viewModel.quote.value?.quote?.toValue)

        var confirmCalls = 0
        viewModel.swap { confirmCalls++ }
        awaitCondition { viewModel.uiState.value.isTransferLoading }

        quotesFlow.emit(quotesState.copy(items = listOf(mockSwapperQuote(toValue = BigInteger("2600000")))))
        advanceUntilIdle()

        assertTrue(viewModel.uiState.value.isTransferLoading)
        assertEquals(BigInteger("2500000"), viewModel.quote.value?.quote?.toValue)
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
            mockSession(wallet = wallet)
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.NoQuoteAvailable()

        val viewModel = createViewModel(
            swapSavedState()
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.errorText != null }

        assertEquals(InfoSheetEntity.NoQuoteInfo, viewModel.uiState.value.errorInfo)
        assertEquals(BigInteger("2500000"), viewModel.quote.value?.quote?.toValue)
    }

    @Test
    fun `a transfer failure Core cannot retry leaves the button tappable and retries the transfer`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet)
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
        assertEquals(BigInteger("2500000"), viewModel.quote.value?.quote?.toValue)

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
            mockSession(wallet = wallet)
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.NoQuoteAvailable()

        val viewModel = createViewModel(
            swapSavedState()
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
            mockSession(wallet = wallet)
        )

        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(
            swapSavedState()
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
            swapSavedState()
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
            mockSession(wallet = wallet)
        )
        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(
            swapSavedState()
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
            mockSession(wallet = wallet)
        )
        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(
            swapSavedState()
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
        every { SwapDetailsUIModelFactory.create(any()) } returns SwapDetailsUIModel(
            rows = emptyList(),
            provider = SwapProviderUIModel(
                id = SwapProvider.UNISWAP_V3,
                title = "Uniswap v3",
                icon = "",
            ),
            rate = AssetRatePair(forward = "1 SOL = 2.5 USDC", reverse = "1 USDC = 0.4 SOL"),
            priceImpact = SwapPriceImpactUIModel(
                type = SwapPriceImpactType.HIGH,
                displayText = "-15%",
                warningText = "High price impact",
                isHigh = true,
                showsInSummary = true,
            ),
            minimumReceive = "2.1 USDC",
            slippageText = "0.5%",
            slippageBps = 50u,
            selectedSlippage = 50u,
        )

        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet)
        )

        var swapCalls = 0
        stubBuildConfirmInput { swapCalls += 1 }

        val viewModel = createViewModel(
            swapSavedState()
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
    fun `amount above the balance blocks the button before any quote`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        viewModel.payValue.setTextAndPlaceCursorAtEnd("2")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.transfer_insufficient_balance }

        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)

        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.wallet_swap }
    }

    @Test
    fun `minimum amount is offered only when the balance covers it`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, quotesFlow, SwapperException.InputAmountException("2000000000"))
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.transfer_insufficient_balance }
        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)

        failQuote(viewModel, quotesFlow, SwapperException.InputAmountException("500000000"))
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.swap_use_minimum_amount }
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)

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

    @Test
    fun `only retryable quote failures offer a retry`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any(), any()) } returns quotesFlow

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, quotesFlow, SwapperException.NoQuoteAvailable())
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.common_try_again }
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)

        failQuote(viewModel, quotesFlow, SwapperException.NoAvailableProvider())
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.wallet_swap }
        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)
    }

    private suspend fun failQuote(
        viewModel: SwapViewModel,
        quotesFlow: MutableSharedFlow<SwapQuotesResult?>,
        error: Throwable,
    ) {
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

    private suspend fun seedReadyQuote(
        viewModel: SwapViewModel,
        quotesFlow: MutableSharedFlow<SwapQuotesResult?>,
        quote: SwapperQuote = mockSwapperQuote(),
    ): SwapQuotesResult {
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
