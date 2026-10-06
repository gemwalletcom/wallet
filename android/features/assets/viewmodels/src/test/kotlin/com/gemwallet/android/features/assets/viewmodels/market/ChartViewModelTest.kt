package com.gemwallet.android.features.assets.viewmodels.market

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.store.queries.PriceQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetMarket
import com.gemwallet.android.testkit.mockGemChart
import com.gemwallet.android.testkit.mockGemChartData
import com.gemwallet.android.testkit.mockPrice
import com.gemwallet.android.testkit.mockPriceAlert
import com.gemwallet.android.ui.models.StateViewType
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceData
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemChartInput
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemChartRate
import uniffi.gemstone.GemChartRequest
import uniffi.gemstone.GemChartResult
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemChartSession
import uniffi.gemstone.GemChartView
import uniffi.gemstone.GemChartZoom
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemListSectionFooter
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemLoadState

@OptIn(ExperimentalCoroutinesApi::class)
class ChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(testDispatcher)

    private val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6u, type = AssetType.SPL)
    private val currencyFlow = MutableStateFlow(Currency.USD)
    private val priceDataFlow = MutableStateFlow<PriceData?>(PriceData(asset = asset, priceAlerts = emptyList(), links = emptyList()))
    private val walletAssetsFlow = MutableStateFlow<List<AssetData>>(emptyList())
    private val viewModels = mutableListOf<ViewModel>()

    private val getCurrentCurrency = mockk<GetCurrentCurrency>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }
    private val getWalletAssets = mockk<GetWalletAssets>(relaxed = true) {
        every { this@mockk.invoke() } returns walletAssetsFlow
    }
    private val priceQuery = mockk<PriceQuery> { every { this@mockk(asset.id) } returns priceDataFlow }
    private val chartService = mockk<GemChartService>(relaxed = true)
    private var savedPeriod = ChartPeriod.Day
    private var sections = emptyList<GemListSection>()

    @Before
    fun setUp() {
        every { chartService.newSession() } answers {
            GemChartSession(savedPeriod.toGem(), currencyFlow.value.toGem(), rate = GemChartRate.Pending, chart = null, error = null, isLoading = true, isRefreshing = false, zoom = GemChartZoom(scale = 1.0, offset = 0.0))
        }
        every { chartService.viewState(any(), any()) } answers {
            val session = firstArg<GemChartSession>()
            GemChartView(
                period = session.period,
                phase = when {
                    session.isLoading -> GemChartPhase.Loading
                    session.chart != null -> GemChartPhase.Data(mockGemChartData(values = session.chart!!.values))
                    else -> GemChartPhase.NoData
                },
                isRefreshing = session.isRefreshing,
                sections = sections,
            )
        }
        coEvery { chartService.load(asset.id.toIdentifier(), any()) } answers {
            val request = secondArg<GemChartRequest>()
            GemChartResult(request = request, rate = 1.0, state = GemLoadState.Data, chart = (request as? GemChartRequest.Chart)?.let { mockGemChart() })
        }
    }

    @After
    fun tearDown() {
        viewModels.forEach { viewModel ->
            val job = viewModel.viewModelScope.coroutineContext.job
            job.cancel()
            while (!job.isCompleted) {
                testDispatcher.scheduler.advanceUntilIdle()
            }
        }
        viewModels.clear()
    }

    @Test
    fun `opening the chart asks for the rate and then the saved period`() = runTest(testDispatcher) {
        savedPeriod = ChartPeriod.Month
        val chart = mockGemChart(values = (1..3).map { uniffi.gemstone.ChartDateValue(date = it * 60_000L, value = it.toDouble()) }, baseValue = 1.0)
        coEvery { chartService.load(asset.id.toIdentifier(), GemChartRequest.Chart(ChartPeriod.Month.toGem(), Currency.USD.toGem())) } returns GemChartResult(
            request = GemChartRequest.Chart(ChartPeriod.Month.toGem(), Currency.USD.toGem()),
            rate = 1.0,
            state = GemLoadState.Data,
            chart = chart,
        )

        val viewModel = createViewModel()
        val state = viewModel.chartUIState.first { it.chart is StateViewType.Data }

        assertEquals(ChartPeriod.Month, state.period)
        assertEquals(chart.values.size, (state.chart as StateViewType.Data).data.values.size)
        coVerify(exactly = 1) { chartService.load(asset.id.toIdentifier(), GemChartRequest.Rate(Currency.USD.toGem())) }
        coVerify(exactly = 1) { chartService.load(asset.id.toIdentifier(), GemChartRequest.Chart(ChartPeriod.Month.toGem(), Currency.USD.toGem())) }
    }

    @Test
    fun `selecting a period loads it without the refresh indicator and without asking for the rate again`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart is StateViewType.Data }
        backgroundScope.launch { viewModel.isRefreshing.collect {} }
        backgroundScope.launch { viewModel.chartUIState.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        val inFlight = CompletableDeferred<Unit>()
        val week = GemChartRequest.Chart(ChartPeriod.Week.toGem(), Currency.USD.toGem())
        coEvery { chartService.load(asset.id.toIdentifier(), week) } coAnswers {
            inFlight.await()
            GemChartResult(request = week, rate = 1.0, state = GemLoadState.Data, chart = mockGemChart())
        }
        viewModel.setPeriod(ChartPeriod.Week)
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(ChartPeriod.Week, viewModel.chartUIState.value.period)
        assertTrue(viewModel.chartUIState.value.chart is StateViewType.Loading)
        assertEquals(false, viewModel.isRefreshing.value)

        inFlight.complete(Unit)
        testDispatcher.scheduler.advanceUntilIdle()

        assertTrue(viewModel.chartUIState.value.chart is StateViewType.Data)
        coVerify(exactly = 1) { chartService.load(asset.id.toIdentifier(), GemChartRequest.Rate(Currency.USD.toGem())) }
    }

    @Test
    fun `a pull to refresh keeps the chart that is already drawn`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart is StateViewType.Data }
        backgroundScope.launch { viewModel.isRefreshing.collect {} }
        backgroundScope.launch { viewModel.chartUIState.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        val inFlight = CompletableDeferred<Unit>()
        val day = GemChartRequest.Chart(ChartPeriod.Day.toGem(), Currency.USD.toGem())
        coEvery { chartService.load(asset.id.toIdentifier(), day) } coAnswers {
            inFlight.await()
            GemChartResult(request = day, rate = 1.0, state = GemLoadState.Data, chart = mockGemChart())
        }
        viewModel.refresh()
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(true, viewModel.isRefreshing.value)
        assertTrue(viewModel.chartUIState.value.chart is StateViewType.Data)

        inFlight.complete(Unit)
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(false, viewModel.isRefreshing.value)
        coVerify(exactly = 2) { chartService.load(asset.id.toIdentifier(), day) }
    }

    @Test
    fun `a pinch during a refresh keeps the load that is in flight`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart is StateViewType.Data }
        backgroundScope.launch { viewModel.chartUIState.collect {} }
        backgroundScope.launch { viewModel.isRefreshing.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        val inFlight = CompletableDeferred<Unit>()
        val day = GemChartRequest.Chart(ChartPeriod.Day.toGem(), Currency.USD.toGem())
        coEvery { chartService.load(asset.id.toIdentifier(), day) } coAnswers {
            inFlight.await()
            GemChartResult(request = day, rate = 1.0, state = GemLoadState.Data, chart = mockGemChart())
        }
        viewModel.refresh()
        testDispatcher.scheduler.advanceUntilIdle()
        repeat(3) {
            viewModel.onZoom(1.5f, 1f)
            testDispatcher.scheduler.advanceUntilIdle()
        }
        inFlight.complete(Unit)
        testDispatcher.scheduler.advanceUntilIdle()

        coVerify(exactly = 2) { chartService.load(asset.id.toIdentifier(), day) }
        assertEquals(false, viewModel.isRefreshing.value)
    }

    @Test
    fun `a stored asset gives the scene its title before any flow emits and its sections once core builds them`() = runTest(testDispatcher) {
        walletAssetsFlow.value = listOf(mockAssetData(asset = asset))
        sections = listOf(section(listOf(GemListRow.Text(GemListRowTitle.TYPE, "SPL"))))

        val viewModel = createViewModel()
        assertEquals(asset.name, viewModel.title.value)

        assertEquals(1, viewModel.sections.first { it.isNotEmpty() }.size)
    }

    @Test
    fun `an asset the wallet does not hold leaves the scene empty until it loads`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        assertTrue(viewModel.sections.value.isEmpty())
        assertEquals("", viewModel.title.value)
        assertEquals(asset.name, viewModel.title.first { it.isNotBlank() })
    }

    @Test
    fun `the stored price, market, alerts and links reach core untouched`() = runTest(testDispatcher) {
        val alert = mockPriceAlert(assetId = asset.id)
        val market = mockAssetMarket(marketCap = 1234.0)
        val link = mockAssetLink()
        priceDataFlow.value = PriceData(asset = asset, price = mockPrice(price = 2.5), priceAlerts = listOf(alert), market = market, links = listOf(link))

        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.sections.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        verify {
            chartService.viewState(
                any(),
                GemChartInput(asset = asset.toGem(), price = mockPrice(price = 2.5).toGem(), market = market.toGem(), priceAlerts = listOf(alert.toGem()), links = listOf(link.toGem())),
            )
        }
    }

    private fun createViewModel(): ChartViewModel = ChartViewModel(
        getCurrentCurrency = getCurrentCurrency,
        priceQuery = priceQuery,
        getWalletAssets = getWalletAssets,
        chartService = chartService,
        assetId = asset.id,
        observeRefreshInterval = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
        context = mockk(relaxed = true),
    ).also(viewModels::add)

    private fun section(rows: List<GemListRow>, title: GemListSectionTitle = GemListSectionTitle.NONE) = GemListSection(title, GemListSectionFooter.NONE, rows)
}
