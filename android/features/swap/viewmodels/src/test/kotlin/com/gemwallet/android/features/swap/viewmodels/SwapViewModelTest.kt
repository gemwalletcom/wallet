package com.gemwallet.android.features.swap.viewmodels

import android.content.Context
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockBalance
import com.gemwallet.android.testkit.mockGemSwapSession
import com.gemwallet.android.testkit.mockGemSwapTransfer
import com.gemwallet.android.testkit.mockPrice
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockSwapQuote
import com.gemwallet.android.testkit.mockSwapperQuote
import com.gemwallet.android.testkit.mockSwapperQuoteAsset
import com.gemwallet.android.testkit.mockSwapperQuoteRequest
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
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
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
    private val solInfo = mockAssetData(
        account = mockAccount(chain = Chain.Solana),
        asset = solAsset,
        balance = mockBalance(available = BigInteger("1000000000")),
    )
    private val usdcInfo = mockAssetData(account = mockAccount(chain = Chain.Solana), asset = usdcAsset)

    private val getSession = mockk<GetSession>(relaxed = true) {
        every { this@mockk() } returns MutableStateFlow(null)
    }
    private val walletId = mockWalletId()
    private val getCurrentWalletId = mockk<GetCurrentWalletId> {
        every { this@mockk() } returns flowOf(walletId)
    }
    private val assetQuery = mockk<AssetQuery>(relaxed = true) {
        every { this@mockk(walletId.id, solAsset.id) } returns flowOf(solInfo)
        every { this@mockk(walletId.id, usdcAsset.id) } returns flowOf(usdcInfo)
    }
    private val quoteAnswers = Channel<Result<List<SwapperQuote>>>(Channel.UNLIMITED)
    private val swapQuoteService = mockk<GemSwapQuoteServiceInterface>(relaxed = true) {
        coEvery { getQuotes(any(), any(), any(), any(), any()) } coAnswers { quoteAnswers.receive().getOrThrow() }
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
        clearMocks(getSession, assetQuery)
        clearMocks(swapQuoteService, answers = false)
        every { getSession() } returns MutableStateFlow(null)
        every { assetQuery(walletId.id, solAsset.id) } returns flowOf(solInfo)
        every { assetQuery(walletId.id, usdcAsset.id) } returns flowOf(usdcInfo)
    }

    @After
    fun tearDown() = runTest(testDispatcher) {
        createdViewModels.forEach { it.viewModelScope.coroutineContext.job.cancelAndJoin() }
        createdViewModels.clear()
        Dispatchers.resetMain()
    }

    private fun createViewModel(savedStateHandle: SavedStateHandle) = SwapViewModel(
        getCurrentWalletId = getCurrentWalletId,
        assetQuery = assetQuery,
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
        assertEquals(solAsset.id, viewModel.payAsset.value?.asset?.id)
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
    fun `quotes are requested for the typed amount, the pair and the saved slippage`() = runTest(testDispatcher) {
        every { swapQuoteService.slippageBps() } returns 200u
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "0.5")

        coVerify(exactly = 1) { swapQuoteService.getQuotes(solAsset.toGem(), usdcAsset.toGem(), BigInteger("500000000"), false, 200u) }
    }

    @Test
    fun `no quotes are requested without an amount`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        viewModel.setRefreshEnabled(true)
        advanceUntilIdle()

        coVerify(exactly = 0) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `an amount changed during the debounce requests quotes once, for the new amount`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)

        viewModel.payValue.setTextAndPlaceCursorAtEnd("0.5")
        Snapshot.sendApplyNotifications()
        advanceTimeBy(GemConstants.swapQuoteDebounce.inWholeMilliseconds / 2)
        requestQuote(viewModel, "0.25")

        coVerify(exactly = 1) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
        coVerify(exactly = 1) { swapQuoteService.getQuotes(any(), any(), BigInteger("250000000"), any(), any()) }
    }

    @Test
    fun `an amount changed while quotes load drops the stale request without an error`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)

        requestQuote(viewModel, "0.5")
        requestQuote(viewModel, "0.25")

        coVerify(exactly = 2) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
        assertTrue(viewModel.uiState.value.isQuoteLoading)
        assertNull(viewModel.uiState.value.errorText)
    }

    @Test
    fun `a quote answered after the amount is cleared is not shown`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)

        requestQuote(viewModel, "0.5")
        requestQuote(viewModel, "")
        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2500000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        advanceUntilIdle()

        assertNull(viewModel.swapDetails.value)
        assertEquals("", viewModel.receiveValue.text.toString())
    }

    @Test
    fun `the same amount in another notation requests no new quotes`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "1")
        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2500000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        runCurrent()

        viewModel.payValue.setTextAndPlaceCursorAtEnd("1.0")
        Snapshot.sendApplyNotifications()
        advanceTimeBy(GemConstants.swapQuoteDebounce.inWholeMilliseconds * 2)

        coVerify(exactly = 1) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
    }

    @Test
    fun `picking the receive asset for a typed amount requests quotes at once`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState(to = null))
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "0.5")

        pairSelection = GemSwapPairSelection(solAsset.id.toIdentifier(), usdcAsset.id.toIdentifier())
        viewModel.onSelect(SwapItemType.Receive, usdcAsset.id)
        runCurrent()

        coVerify(exactly = 1) { swapQuoteService.getQuotes(solAsset.toGem(), usdcAsset.toGem(), BigInteger("500000000"), false, null) }
    }

    @Test
    fun `a quote is refreshed only after the refresh interval`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "1")
        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2500000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        runCurrent()

        advanceTimeBy(GemConstants.swapQuoteRefreshInterval.inWholeMilliseconds - 1)
        runCurrent()
        coVerify(exactly = 1) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }

        advanceUntilIdle()
        coVerify(exactly = 2) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `a quote error schedules no automatic retry`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, SwapperException.ComputeQuoteException("boom"))

        coVerify(exactly = 1) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
        assertNotNull(viewModel.uiState.value.errorText)
    }

    @Test
    fun `automatic refresh stops in background and resumes in foreground`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "1")
        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2500000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        runCurrent()

        viewModel.setRefreshEnabled(false)
        advanceUntilIdle()
        coVerify(exactly = 1) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }

        viewModel.setRefreshEnabled(true)
        advanceUntilIdle()
        coVerify(exactly = 2) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `a quote refresh in flight when the swap starts does not replace swapping state`() = runTest(testDispatcher) {
        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        seedReadyQuote(viewModel)
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)

        viewModel.setRefreshEnabled(true)
        advanceUntilIdle()
        val confirmed = mutableListOf<ConfirmTransferInput>()
        viewModel.swap { confirmed += it }
        awaitCondition { viewModel.uiState.value.isTransferLoading }

        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2600000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        advanceUntilIdle()

        assertTrue(viewModel.uiState.value.isTransferLoading)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)
        assertEquals(0, confirmed.size)

        confirmInputGate.complete(Unit)
        awaitCondition { confirmed.size == 1 }
    }

    @Test
    fun `transfer data error keeps quote visible and routes retry through transfer state`() = runTest(testDispatcher) {
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.NoQuoteAvailable()

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.errorText != null }

        assertEquals(GemInfoTopic.NoQuote.infoSheet(), viewModel.uiState.value.errorInfo)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)
    }

    @Test
    fun `a transfer failure Core cannot retry leaves the button tappable and retries the transfer`() = runTest(testDispatcher) {
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.InvalidRoute()

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        seedReadyQuote(viewModel)

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.errorText != null }

        val state = viewModel.uiState.value
        assertEquals(R.string.wallet_swap, state.actionTitle)
        assertEquals(ButtonState.Enabled, state.buttonState)
        assertEquals(2.5, viewModel.swapDetails.value?.provider?.amount?.value)

        coEvery { swapQuoteService.getTransfer(any()) } returns mockGemSwapTransfer(recipient = solInfo.account.address)

        var confirmed: GemTransferData? = null
        viewModel.onPrimaryAction(
            onConfirm = { confirmed = it.data },
            onShowPriceImpactWarning = {},
            authorize = { it() },
        )
        awaitCondition { confirmed != null }

        assertEquals(solInfo.account.address, confirmed?.recipient?.address)
    }

    @Test
    fun `quote changing actions clear transfer error state`() = runTest(testDispatcher) {
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(
            mockSession(wallet = wallet),
        )
        coEvery { swapQuoteService.getTransfer(any()) } throws SwapperException.NoQuoteAvailable()

        val viewModel = createViewModel(
            swapSavedState(),
        )
        advanceUntilIdle()

        seedReadyQuote(viewModel)
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
        val confirmInputGate = CompletableDeferred<Unit>()
        stubBuildConfirmInput { confirmInputGate.await() }

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "1")
        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2500000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        runCurrent()

        viewModel.swap {}
        awaitCondition { viewModel.uiState.value.isTransferLoading }
        confirmInputGate.complete(Unit)
        awaitCondition { viewModel.uiState.value.buttonState == ButtonState.Enabled && viewModel.uiState.value.errorText == null }
        advanceUntilIdle()
        coVerify(exactly = 1) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }

        viewModel.setRefreshEnabled(false)
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        advanceUntilIdle()
        coVerify(exactly = 2) { swapQuoteService.getQuotes(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `an automatic quote refresh shows quote loading`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "1")
        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2500000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        runCurrent()
        assertEquals(ButtonState.Enabled, viewModel.uiState.value.buttonState)
        assertNull(viewModel.uiState.value.errorText)

        advanceUntilIdle()

        assertTrue(viewModel.uiState.value.isQuoteLoading)
        assertEquals("2.5", viewModel.receiveValue.text.toString())
    }

    @Test
    fun `confirm callback runs before transfer loading clears`() = runTest(testDispatcher) {
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

        seedReadyQuote(viewModel)

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

        seedReadyQuote(viewModel)

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
        val wallet = mockWallet(accounts = listOf(mockAccount(chain = solAsset.id.chain)))
        every { getSession() } returns MutableStateFlow(mockSession(wallet = wallet))

        var swapCalls = 0
        stubBuildConfirmInput { swapCalls += 1 }

        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()
        seedReadyQuote(viewModel)

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
        every { assetQuery(walletId.id, solAsset.id) } returns flowOf(solInfo.copy(price = mockPrice(price = 100.0)))
        every { assetQuery(walletId.id, usdcAsset.id) } returns flowOf(usdcInfo.copy(price = mockPrice(price = 1.0)))

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

        seedReadyQuote(viewModel)

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
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, SwapperException.InputAmountException("500000000"))
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.swap_use_minimum_amount }

        viewModel.onPrimaryAction(onConfirm = {}, onShowPriceImpactWarning = {}, authorize = { it() })
        advanceUntilIdle()

        assertEquals("0.5", viewModel.payValue.text.toString())
    }

    @Test
    fun `small minimum amount uses editable decimal notation`() = runTest(testDispatcher) {
        val viewModel = createViewModel(swapSavedState())
        advanceUntilIdle()

        failQuote(viewModel, SwapperException.InputAmountException("1"))
        awaitCondition { viewModel.uiState.value.actionTitle == R.string.swap_use_minimum_amount }

        viewModel.onPrimaryAction(onConfirm = {}, onShowPriceImpactWarning = {}, authorize = { it() })
        advanceUntilIdle()

        assertEquals("0.000000001", viewModel.payValue.text.toString())
    }

    private fun failQuote(viewModel: SwapViewModel, error: SwapperException) {
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "1")
        quoteAnswers.trySend(Result.failure(error))
        testDispatcher.scheduler.advanceUntilIdle()
    }

    private fun requestQuote(viewModel: SwapViewModel, text: String) {
        viewModel.payValue.setTextAndPlaceCursorAtEnd(text)
        Snapshot.sendApplyNotifications()
        testDispatcher.scheduler.advanceUntilIdle()
    }

    private fun stubBuildConfirmInput(beforeReturn: suspend () -> Unit = {}) {
        coEvery { swapQuoteService.getTransfer(any()) } coAnswers {
            beforeReturn()
            val quote = firstArg<SwapperQuote>()
            mockGemSwapTransfer(
                quote = mockSwapQuote(fromValue = quote.fromValue, toValue = quote.toValue, useMaxAmount = quote.request.options.useMaxAmount),
                recipient = solInfo.account.address,
                value = quote.fromValue,
                useMaxAmount = quote.request.options.useMaxAmount,
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

    private fun seedReadyQuote(viewModel: SwapViewModel) {
        viewModel.setRefreshEnabled(true)
        requestQuote(viewModel, "1")
        quoteAnswers.trySend(Result.success(listOf(mockSwapperQuote(fromValue = BigInteger("1000000000"), toValue = BigInteger("2500000"), request = mockSwapperQuoteRequest(toAsset = mockSwapperQuoteAsset(decimals = 6u))))))
        testDispatcher.scheduler.runCurrent()
        viewModel.setRefreshEnabled(false)
        awaitCondition { viewModel.uiState.value.buttonState == ButtonState.Enabled && viewModel.uiState.value.errorText == null }
    }
}
