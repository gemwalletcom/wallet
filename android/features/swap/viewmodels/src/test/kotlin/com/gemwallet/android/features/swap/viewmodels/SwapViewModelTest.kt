package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.swap.SwapRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.swap.viewmodels.cases.QuoteRequester
import com.gemwallet.android.features.swap.viewmodels.models.QuoteRequestParams
import com.gemwallet.android.features.swap.viewmodels.models.QuotesState
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.features.swap.viewmodels.models.SwapState
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import io.mockk.coEvery
import io.mockk.clearMocks
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.unmockkObject
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
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
import java.math.BigDecimal

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
    private val swapRepository = mockk<SwapRepository>(relaxed = true)
    private val quoteRequester = mockk<QuoteRequester>(relaxed = true) {
        every { requestQuotes(any(), any(), any(), any(), any()) } returns emptyFlow()
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        clearMocks(sessionRepository, assetsRepository, swapRepository, quoteRequester)
        every { sessionRepository.session() } returns MutableStateFlow(null)
        every { assetsRepository.getAssetInfo(solAsset.id) } returns flowOf(solInfo)
        every { assetsRepository.getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo)
        every { quoteRequester.requestQuotes(any(), any(), any(), any(), any()) } returns emptyFlow()
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
        unmockkObject(SwapDetailsUIModelFactory)
    }

    private fun createViewModel(savedStateHandle: SavedStateHandle) = SwapViewModel(
        sessionRepository = sessionRepository,
        assetsRepository = assetsRepository,
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
        mockkObject(SwapDetailsUIModelFactory)
        every { SwapDetailsUIModelFactory.create(any()) } returns mockk(relaxed = true)

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

        viewModel.payValue.setTextAndPlaceCursorAtEnd("1")
        Snapshot.sendApplyNotifications()
        advanceUntilIdle()
        awaitCondition { viewModel.uiSwapScreenState.value == SwapState.GetQuote }

        val quotesState = QuotesState(
            items = listOf(mockQuote()),
            requestKey = QuoteRequestParams(BigDecimal.ONE, solInfo, usdcInfo).key,
            pay = solInfo,
            receive = usdcInfo,
        )
        quotesFlow.emit(quotesState)
        advanceUntilIdle()

        assertEquals(SwapState.Ready, viewModel.uiSwapScreenState.value)

        var confirmCalls = 0
        viewModel.swap { confirmCalls++ }
        awaitCondition { viewModel.uiSwapScreenState.value == SwapState.Swapping }

        quotesFlow.emit(quotesState.copy(items = listOf(mockQuote(toValue = "2600000"))))
        advanceUntilIdle()

        assertEquals(SwapState.Swapping, viewModel.uiSwapScreenState.value)
        assertEquals(0, confirmCalls)

        quoteDataGate.complete(Unit)
        awaitCondition { confirmCalls == 1 }
    }

    private fun awaitCondition(timeoutMs: Long = 2_000, condition: () -> Boolean) {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (!condition() && System.currentTimeMillis() < deadline) {
            testDispatcher.scheduler.advanceUntilIdle()
            Thread.sleep(10)
        }
        assertTrue("condition not met within ${timeoutMs}ms", condition())
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
