package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.assets.cases.EnableAsset
import com.gemwallet.android.application.swap.cases.BuildSwapConfirmParams
import com.gemwallet.android.application.swap.cases.RequestSwapQuotes
import com.gemwallet.android.application.swap.cases.SwapNoQuoteException
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestKey
import com.gemwallet.android.application.swap.cases.SwapQuoteRequestParams
import com.gemwallet.android.application.swap.cases.SwapQuotesResult
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.swap.AssetRatePair
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.swap.viewmodels.models.SwapActionState
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.testkit.mockSwapParams
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapPriceImpactUIModel
import com.gemwallet.android.ui.models.swap.SwapProviderUIModel
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.swap.SwapPriceImpactType
import com.wallet.core.primitives.swap.SwapQuoteDataType
import io.mockk.clearMocks
import io.mockk.coEvery
import uniffi.gemstone.GemSwapPairSuggestion
import uniffi.gemstone.GemSwapServiceInterface
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.slot
import io.mockk.unmockkObject
import java.math.BigDecimal
import java.math.BigInteger
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.SwapperOptions
import uniffi.gemstone.SwapperProvider
import uniffi.gemstone.SwapperProviderData
import uniffi.gemstone.SwapperProviderMode
import uniffi.gemstone.SwapperProviderType
import uniffi.gemstone.SwapperQuote
import uniffi.gemstone.SwapperQuoteAsset
import uniffi.gemstone.SwapperQuoteRequest
import uniffi.gemstone.SwapperRoute
import uniffi.gemstone.SwapperSlippage
import uniffi.gemstone.SwapperSlippageMode
import uniffi.gemstone.GemSwapButtonAction
import uniffi.gemstone.GemSwapQuoteService
import uniffi.gemstone.SwapperException

@OptIn(ExperimentalCoroutinesApi::class)
class SwapViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    private val solAsset = mockAssetSolana()
    private val usdcAsset = mockAssetSolanaUSDC()
    private val solInfo = mockAssetInfo(
        asset = solAsset,
        balance = AssetBalance.create(solAsset, available = "1000000000"),
    )
    private val usdcInfo = mockAssetInfo(asset = usdcAsset)

    private val getSession = mockk<GetSession>(relaxed = true) {
        every { this@mockk() } returns MutableStateFlow(null)
    }
    private val getAssetInfo = mockk<GetAssetInfo>(relaxed = true) {
        every { this@mockk(solAsset.id) } returns flowOf(solInfo)
        every { this@mockk(usdcAsset.id) } returns flowOf(usdcInfo)
    }
    private val enableAsset = mockk<EnableAsset>(relaxed = true)
    private val buildSwapConfirmParams = mockk<BuildSwapConfirmParams>(relaxed = true)
    private val userConfig = mockk<UserConfig>(relaxed = true) {
        every { swapSlippageBps() } returns flowOf(null)
    }
    private val requestSwapQuotes = mockk<RequestSwapQuotes>(relaxed = true)
    private val swapService = mockk<GemSwapServiceInterface> {
        coEvery { suggestPair(any(), any()) } returns null
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        mockkObject(SwapDetailsUIModelFactory)
        clearMocks(getSession, getAssetInfo, buildSwapConfirmParams, requestSwapQuotes)
        every { getSession() } returns MutableStateFlow(null)
        every { getAssetInfo(solAsset.id) } returns flowOf(solInfo)
        every { getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns emptyFlow()
        every { SwapDetailsUIModelFactory.create(any(), any()) } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        unmockkObject(SwapDetailsUIModelFactory)
    }

    private fun createViewModel(savedStateHandle: SavedStateHandle) = SwapViewModel(
        getSession = getSession,
        getAssetInfo = getAssetInfo,
        enableAsset = enableAsset,
        buildSwapConfirmParams = buildSwapConfirmParams,
        userConfig = userConfig,
        swapService = swapService,
        requestSwapQuotes = requestSwapQuotes,
        swapQuoteService = GemSwapQuoteService(),
        savedStateHandle = savedStateHandle,
    )

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
    fun `init keeps an already selected pair and asks for no suggestion`() = runTest(testDispatcher) {
        val savedState = swapSavedState()

        createViewModel(savedState)
        advanceUntilIdle()

        coVerify(exactly = 0) { swapService.suggestPair(any(), any()) }
        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
    }

    @Test
    fun `init applies the suggested pair when the screen opens empty`() = runTest(testDispatcher) {
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(Session(wallet = wallet, currency = Currency.USD))
        coEvery { swapService.suggestPair(wallet.id.id, null) } returns GemSwapPairSuggestion(
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

        coVerify { userConfig.setSwapSlippageBps(200u) }
    }

    @Test
    fun `onSelect updates pay asset from empty state`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

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
        viewModel.onSelect(SwapItemType.Receive, usdcAsset.id)
        advanceUntilIdle()

        assertEquals("1.5", viewModel.payValue.text.toString())
        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
    }

    @Test
    fun `selecting same receive asset clears pay asset and amount`() = runTest(testDispatcher) {
        val savedState = swapSavedState()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        viewModel.onSelect(SwapItemType.Receive, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.ToAssetId.key))
        assertNull("pay must be cleared when receive matches it", savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertEquals("", viewModel.payValue.text.toString())
    }

    @Test
    fun `selecting same pay asset clears receive`() = runTest(testDispatcher) {
        val savedState = swapSavedState(
            from = usdcAsset.id.toIdentifier(),
            to = solAsset.id.toIdentifier(),
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Pay, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>(RouteArgument.FromAssetId.key))
        assertNull(savedState.get<String?>(RouteArgument.ToAssetId.key))
    }

    @Test
    fun `quote refresh does not replace swapping state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )

        val confirmParamsGate = CompletableDeferred<Unit>()
        stubBuildConfirmParams { confirmParamsGate.await() }

        val savedState = swapSavedState()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        val quotesState = seedReadyQuote(viewModel, quotesFlow)
        assertEquals(SwapActionState.Ready, viewModel.uiState.value.action)
        assertEquals("2500000", viewModel.quote.value?.quote?.toValue)

        var confirmCalls = 0
        viewModel.swap { confirmCalls++ }
        awaitCondition { viewModel.uiState.value.action == SwapActionState.TransferLoading }

        quotesFlow.emit(quotesState.copy(items = listOf(mockQuote(toValue = "2600000"))))
        advanceUntilIdle()

        assertEquals(SwapActionState.TransferLoading, viewModel.uiState.value.action)
        assertEquals("2500000", viewModel.quote.value?.quote?.toValue)
        assertEquals(0, confirmCalls)

        confirmParamsGate.complete(Unit)
        awaitCondition { confirmCalls == 1 }
    }

    @Test
    fun `transfer data error keeps quote visible and routes retry through transfer state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        coEvery { buildSwapConfirmParams(any(), any(), any()) } throws SwapNoQuoteException()

        val viewModel = createViewModel(
            swapSavedState()
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.action is SwapActionState.TransferError }

        val action = viewModel.uiState.value.action as SwapActionState.TransferError
        assertTrue(action.error is SwapError.NoQuote)
        assertEquals("2500000", viewModel.quote.value?.quote?.toValue)
    }

    @Test
    fun `quote changing actions clear transfer error state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        coEvery { buildSwapConfirmParams(any(), any(), any()) } throws SwapNoQuoteException()

        val viewModel = createViewModel(
            swapSavedState()
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)
        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.action is SwapActionState.TransferError }

        viewModel.setProvider(SwapperProvider.UNISWAP_V3)
        advanceUntilIdle()

        assertEquals(SwapActionState.Ready, viewModel.uiState.value.action)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.action is SwapActionState.TransferError }

        viewModel.payValue.setTextAndPlaceCursorAtEnd("2")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.action == SwapActionState.QuoteLoading }
    }

    @Test
    fun `quote refresh stays paused after confirm handoff until screen restarts`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        val refreshEnabledFlow = slot<Flow<Boolean>>()
        every {
            requestSwapQuotes.invoke(any(), any(), capture(refreshEnabledFlow), any(), any())
        } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )

        val confirmParamsGate = CompletableDeferred<Unit>()
        stubBuildConfirmParams { confirmParamsGate.await() }

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
        awaitCondition { viewModel.uiState.value.action == SwapActionState.TransferLoading }
        confirmParamsGate.complete(Unit)
        awaitCondition { viewModel.uiState.value.action == SwapActionState.Ready }
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
        val onFetchStarted = slot<(SwapQuoteRequestKey) -> Unit>()
        every {
            requestSwapQuotes.invoke(any(), any(), any(), capture(onFetchStarted), any())
        } returns quotesFlow

        val viewModel = createViewModel(
            swapSavedState()
        )
        advanceUntilIdle()

        val seededQuotes = seedReadyQuote(viewModel, quotesFlow)
        assertEquals(SwapActionState.Ready, viewModel.uiState.value.action)

        onFetchStarted.captured(seededQuotes.requestKey)
        advanceUntilIdle()

        assertEquals(SwapActionState.QuoteLoading, viewModel.uiState.value.action)
    }

    @Test
    fun `confirm callback runs before transfer loading clears`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        val confirmParamsGate = CompletableDeferred<Unit>()
        stubBuildConfirmParams { confirmParamsGate.await() }

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
        confirmParamsGate.complete(Unit)
        awaitCondition { !viewModel.uiState.value.isTransferLoading }

        assertTrue(wasTransferLoadingOnConfirm)
        assertEquals(SwapActionState.Ready, viewModel.uiState.value.action)
    }

    @Test
    fun `confirm params keep frozen from amount while transfer is in flight`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        val confirmParamsGate = CompletableDeferred<Unit>()
        stubBuildConfirmParams { confirmParamsGate.await() }

        val viewModel = createViewModel(
            swapSavedState()
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        var confirmParams: ConfirmParams.SwapParams? = null
        viewModel.swap { params ->
            confirmParams = params as ConfirmParams.SwapParams
        }
        awaitCondition { viewModel.uiState.value.isTransferLoading }

        viewModel.payValue.setTextAndPlaceCursorAtEnd("2")
        confirmParamsGate.complete(Unit)
        awaitCondition { confirmParams != null }

        assertEquals(BigInteger("1000000000"), confirmParams?.amount)
    }

    @Test
    fun `onPrimaryAction shows price impact warning before swap`() = runTest(testDispatcher) {
        every { SwapDetailsUIModelFactory.create(any(), any()) } returns SwapDetailsUIModel(
            provider = SwapProviderUIModel(
                id = SwapperProvider.UNISWAP_V3,
                title = "Uniswap v3",
                icon = "",
            ),
            rate = AssetRatePair(forward = "1 SOL = 2.5 USDC", reverse = "1 USDC = 0.4 SOL"),
            priceImpact = SwapPriceImpactUIModel(
                type = SwapPriceImpactType.High,
                displayText = "-15%",
                warningText = "High price impact",
                isHigh = true,
            ),
            minimumReceive = "2.1 USDC",
            slippageText = "0.5%",
            slippageBps = 50u,
            selectedSlippage = 50u,
        )

        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )

        var swapCalls = 0
        stubBuildConfirmParams { swapCalls += 1 }

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
        )
        advanceUntilIdle()

        assertEquals(1, showWarningCalls)
        assertEquals(0, swapCalls)
        assertEquals(0, confirmCalls)
        assertEquals(SwapActionState.Ready, viewModel.uiState.value.action)
    }

    @Test
    fun `amount above the balance blocks the button before any quote`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        viewModel.payValue.setTextAndPlaceCursorAtEnd("2")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.buttonAction == GemSwapButtonAction.InsufficientBalance }

        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)

        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.buttonAction == GemSwapButtonAction.Swap }
    }

    @Test
    fun `minimum amount is offered only when the balance covers it`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, quotesFlow, SwapperException.InputAmountException("2000000000"))
        awaitCondition { viewModel.uiState.value.buttonAction == GemSwapButtonAction.InsufficientBalance }
        assertEquals(ButtonState.Disabled, viewModel.uiState.value.buttonState)

        failQuote(viewModel, quotesFlow, SwapperException.InputAmountException("500000000"))
        awaitCondition { viewModel.uiState.value.buttonAction == GemSwapButtonAction.UseMinimumAmount("500000000") }
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)

        viewModel.onPrimaryAction(onConfirm = {}, onShowPriceImpactWarning = {})
        advanceUntilIdle()

        assertEquals("0.5", viewModel.payValue.text.toString())
    }

    @Test
    fun `only retryable quote failures offer a retry`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<SwapQuotesResult?>(replay = 1)
        every { requestSwapQuotes.invoke(any(), any(), any(), any(), any()) } returns quotesFlow

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, quotesFlow, SwapperException.NoQuoteAvailable())
        awaitCondition { viewModel.uiState.value.buttonAction == GemSwapButtonAction.RetryQuote }
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)

        failQuote(viewModel, quotesFlow, SwapperException.NoAvailableProvider())
        awaitCondition { viewModel.uiState.value.buttonAction == GemSwapButtonAction.Swap }
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
        quotesFlow.emit(
            SwapQuotesResult(
                requestKey = SwapQuoteRequestParams(BigDecimal.ONE, solInfo, usdcInfo).key,
                pay = solInfo,
                receive = usdcInfo,
                err = error,
            )
        )
        testDispatcher.scheduler.advanceUntilIdle()
    }

    private fun stubBuildConfirmParams(beforeReturn: suspend () -> Unit = {}) {
        coEvery { buildSwapConfirmParams(any(), any(), any()) } coAnswers {
            beforeReturn()
            val quote = firstArg<SwapperQuote>()
            val pay = secondArg<AssetInfo>()
            val receive = thirdArg<AssetInfo>()
            mockSwapParams(
                from = pay.owner!!,
                fromAsset = pay.asset,
                fromAmount = BigInteger(quote.fromValue),
                toAsset = receive.asset,
                toAmount = BigInteger(quote.toValue),
                useMaxAmount = quote.request.options.useMaxAmount,
                toAddress = "0xconfirm",
                dataType = SwapQuoteDataType.Contract,
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
        quote: SwapperQuote = mockQuote(),
    ): SwapQuotesResult {
        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.action == SwapActionState.QuoteLoading }

        val quotesState = SwapQuotesResult(
            items = listOf(quote),
            requestKey = SwapQuoteRequestParams(BigDecimal.ONE, solInfo, usdcInfo).key,
            pay = solInfo,
            receive = usdcInfo,
        )
        quotesFlow.emit(quotesState)
        testDispatcher.scheduler.advanceUntilIdle()
        awaitCondition { viewModel.uiState.value.action == SwapActionState.Ready }
        return quotesState
    }

    private fun mockQuote(
        fromValue: String = "1000000000",
        toValue: String = "2500000",
    ) = SwapperQuote(
        fromValue = fromValue,
        minFromValue = null,
        toValue = toValue,
        data = SwapperProviderData(
            provider = SwapperProviderType(
                id = SwapperProvider.UNISWAP_V3,
                name = "Uniswap",
                protocol = "v3",
                protocolId = "uniswap_v3",
                mode = SwapperProviderMode.OnChain,
                slippageMode = SwapperSlippageMode.EXACT,
            ),
            slippageBps = 50u,
            routes = listOf(
                SwapperRoute(
                    input = solAsset.id.toIdentifier(),
                    output = usdcAsset.id.toIdentifier(),
                    routeData = "0x",
                )
            ),
        ),
        request = SwapperQuoteRequest(
            fromAsset = SwapperQuoteAsset(
                id = solAsset.id.toIdentifier(),
                symbol = solAsset.symbol,
                decimals = solAsset.decimals.toUInt(),
            ),
            toAsset = SwapperQuoteAsset(
                id = usdcAsset.id.toIdentifier(),
                symbol = usdcAsset.symbol,
                decimals = usdcAsset.decimals.toUInt(),
            ),
            walletAddress = solInfo.owner!!.address,
            destinationAddress = usdcInfo.owner!!.address,
            value = fromValue,
            options = SwapperOptions(
                slippage = SwapperSlippage(
                    bps = 50u,
                    mode = SwapperSlippageMode.AUTO,
                ),
                useMaxAmount = false,
            ),
        ),
        etaInSeconds = 30u,
    )
}
