package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.swap.SwapRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.swap.viewmodels.cases.QuoteRequester
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestKey
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestParams
import com.gemwallet.android.features.swap.viewmodels.models.QuotesState
import com.gemwallet.android.features.swap.viewmodels.models.SwapActionState
import com.gemwallet.android.features.swap.viewmodels.models.SwapError
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModel
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapPriceImpactUIModel
import com.gemwallet.android.ui.models.swap.SwapProviderUIModel
import com.gemwallet.android.ui.models.swap.SwapRateUIModel
import io.mockk.coEvery
import io.mockk.clearMocks
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.slot
import io.mockk.unmockkObject
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
import uniffi.gemstone.GemSwapQuoteData
import uniffi.gemstone.GemSwapQuoteDataType
import uniffi.gemstone.SwapperMode
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
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.swap.SwapPriceImpactType
import java.math.BigDecimal
import java.math.BigInteger

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

    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        every { session() } returns MutableStateFlow(null)
    }
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true) {
        every { getAssetInfo(solAsset.id) } returns flowOf(solInfo)
        every { getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo)
    }
    private val enableAsset = mockk<EnableAsset>(relaxed = true)
    private val swapRepository = mockk<SwapRepository>(relaxed = true)
    private val quoteRequester = mockk<QuoteRequester>(relaxed = true) {
        every { requestQuotes(any(), any(), any(), any(), any()) } returns emptyFlow()
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        mockkObject(SwapDetailsUIModelFactory)
        clearMocks(sessionRepository, assetsRepository, swapRepository, quoteRequester)
        every { sessionRepository.session() } returns MutableStateFlow(null)
        every { assetsRepository.getAssetInfo(solAsset.id) } returns flowOf(solInfo)
        every { assetsRepository.getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns emptyFlow()
        every { SwapDetailsUIModelFactory.create(any()) } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        unmockkObject(SwapDetailsUIModelFactory)
    }

    private fun createViewModel(savedStateHandle: SavedStateHandle) = SwapViewModel(
        sessionRepository = sessionRepository,
        assetsRepository = assetsRepository,
        enableAsset = enableAsset,
        swapRepository = swapRepository,
        quoteRequester = quoteRequester,
        savedStateHandle = savedStateHandle,
    )

    @Test
    fun `onSelect updates pay asset from empty state`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Pay, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("from"))
        assertNull(savedState.get<String?>("to"))
        assertEquals(solAsset.id, viewModel.payAsset.value?.id())
    }

    @Test
    fun `onSelect keeps opposite asset when pair differs`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle(
            mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier())
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Receive, usdcAsset.id)
        advanceUntilIdle()

        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>("to"))
        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("from"))
    }

    @Test
    fun `selecting same asset for both pay and receive clears opposite`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle(
            mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier())
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Receive, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("to"))
        assertNull("pay must be cleared when receive matches it", savedState.get<String?>("from"))
    }

    @Test
    fun `selecting same pay asset clears receive`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle(
            mapOf("from" to usdcAsset.id.toIdentifier(), "to" to solAsset.id.toIdentifier())
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Pay, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("from"))
        assertNull(savedState.get<String?>("to"))
    }

    @Test
    fun `quote refresh does not replace swapping state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { sessionRepository.session() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )

        val quoteDataGate = CompletableDeferred<Unit>()
        coEvery { swapRepository.getQuoteData(any(), any()) } coAnswers {
            quoteDataGate.await()
            mockQuoteData()
        }

        val savedState = SavedStateHandle(
            mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier())
        )

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

        quoteDataGate.complete(Unit)
        awaitCondition { confirmCalls == 1 }
    }

    @Test
    fun `transfer data error keeps quote visible and routes retry through transfer state`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { sessionRepository.session() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        coEvery { swapRepository.getQuoteData(any(), any()) } throws IllegalStateException("boom")

        val viewModel = createViewModel(
            SavedStateHandle(mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier()))
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
        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { sessionRepository.session() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        coEvery { swapRepository.getQuoteData(any(), any()) } throws IllegalStateException("boom")

        val viewModel = createViewModel(
            SavedStateHandle(mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier()))
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
        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        val refreshEnabledFlow = slot<Flow<Boolean>>()
        every {
            quoteRequester.requestQuotes(any(), any(), capture(refreshEnabledFlow), any(), any())
        } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { sessionRepository.session() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )

        val quoteDataGate = CompletableDeferred<Unit>()
        coEvery { swapRepository.getQuoteData(any(), any()) } coAnswers {
            quoteDataGate.await()
            mockQuoteData()
        }

        val viewModel = createViewModel(
            SavedStateHandle(mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier()))
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
        quoteDataGate.complete(Unit)
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
        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        val onFetchStarted = slot<(QuoteRequestKey) -> Unit>()
        every {
            quoteRequester.requestQuotes(any(), any(), any(), capture(onFetchStarted), any())
        } returns quotesFlow

        val viewModel = createViewModel(
            SavedStateHandle(mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier()))
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
        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { sessionRepository.session() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        val quoteDataGate = CompletableDeferred<Unit>()
        coEvery { swapRepository.getQuoteData(any(), any()) } coAnswers {
            quoteDataGate.await()
            mockQuoteData()
        }

        val viewModel = createViewModel(
            SavedStateHandle(mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier()))
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        var wasTransferLoadingOnConfirm = false
        viewModel.swap {
            wasTransferLoadingOnConfirm = viewModel.uiState.value.isTransferLoading
        }
        awaitCondition { viewModel.uiState.value.isTransferLoading }
        quoteDataGate.complete(Unit)
        awaitCondition { !viewModel.uiState.value.isTransferLoading }

        assertTrue(wasTransferLoadingOnConfirm)
        assertEquals(SwapActionState.Ready, viewModel.uiState.value.action)
    }

    @Test
    fun `confirm params keep frozen from amount while transfer is in flight`() = runTest(testDispatcher) {
        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { sessionRepository.session() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )
        val quoteDataGate = CompletableDeferred<Unit>()
        coEvery { swapRepository.getQuoteData(any(), any()) } coAnswers {
            quoteDataGate.await()
            mockQuoteData()
        }

        val viewModel = createViewModel(
            SavedStateHandle(mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier()))
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel, quotesFlow)

        var confirmParams: ConfirmParams.SwapParams? = null
        viewModel.swap { params ->
            confirmParams = params as ConfirmParams.SwapParams
        }
        awaitCondition { viewModel.uiState.value.isTransferLoading }

        viewModel.payValue.setTextAndPlaceCursorAtEnd("2")
        quoteDataGate.complete(Unit)
        awaitCondition { confirmParams != null }

        assertEquals(BigInteger("1000000000"), confirmParams?.fromAmount)
    }

    @Test
    fun `onPrimaryAction shows price impact warning before swap`() = runTest(testDispatcher) {
        every { SwapDetailsUIModelFactory.create(any()) } returns SwapDetailsUIModel(
            provider = SwapProviderUIModel(
                id = SwapperProvider.UNISWAP_V3,
                title = "Uniswap v3",
                icon = "",
            ),
            rate = SwapRateUIModel(forward = "1 SOL = 2.5 USDC", reverse = "1 USDC = 0.4 SOL"),
            priceImpact = SwapPriceImpactUIModel(
                type = SwapPriceImpactType.High,
                displayText = "-15%",
                warningText = "High price impact",
                isHigh = true,
            ),
            minimumReceive = "2.1 USDC",
            slippageText = "0.5%",
        )

        val quotesFlow = MutableSharedFlow<QuotesState?>(replay = 1)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns quotesFlow

        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { sessionRepository.session() } returns MutableStateFlow(
            Session(wallet = wallet, currency = Currency.USD)
        )

        var swapCalls = 0
        coEvery { swapRepository.getQuoteData(any(), any()) } coAnswers {
            swapCalls += 1
            mockQuoteData()
        }

        val viewModel = createViewModel(
            SavedStateHandle(mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier()))
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
        quotesFlow: MutableSharedFlow<QuotesState?>,
        quote: SwapperQuote = mockQuote(),
    ): QuotesState {
        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        awaitCondition { viewModel.uiState.value.action == SwapActionState.QuoteLoading }

        val quotesState = QuotesState(
            items = listOf(quote),
            requestKey = QuoteRequestParams(BigDecimal.ONE, solInfo, usdcInfo).key,
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
        toValue = toValue,
        data = SwapperProviderData(
            provider = SwapperProviderType(
                id = SwapperProvider.UNISWAP_V3,
                name = "Uniswap",
                protocol = "v3",
                protocolId = "uniswap_v3",
                mode = SwapperProviderMode.OnChain,
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
            mode = SwapperMode.EXACT_IN,
            options = SwapperOptions(
                slippage = SwapperSlippage(
                    bps = 50u,
                    mode = SwapperSlippageMode.AUTO,
                ),
                fee = null,
                preferredProviders = emptyList(),
                useMaxAmount = false,
            ),
        ),
        etaInSeconds = 30u,
    )

    private fun mockQuoteData() = GemSwapQuoteData(
        to = "0xconfirm",
        dataType = GemSwapQuoteDataType.CONTRACT,
        value = "0",
        data = "0x",
        memo = null,
        approval = null,
        gasLimit = "210000",
    )
}
