package com.gemwallet.android.features.market.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockChartDateValue
import com.gemwallet.android.testkit.mockGemChart
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.job
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemChartService
import uniffi.gemstone.GemChartSession
import uniffi.gemstone.GemServiceException

@OptIn(ExperimentalCoroutinesApi::class)
class ChartValuesViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)
    private val currencyFlow = MutableStateFlow(Currency.USD)
    private val viewModels = mutableListOf<ViewModel>()

    private val getCurrentCurrency = mockk<GetCurrentCurrency>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }
    private val chartService = mockk<GemChartService>(relaxed = true)
    private var savedPeriod = ChartPeriod.Day

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { chartService.newSession() } answers {
            GemChartSession(savedPeriod.toGem(), currencyFlow.value.toGem(), chart = null, error = null, isLoading = true, isRefreshing = false)
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
        Dispatchers.resetMain()
    }

    @Test
    fun `historical chart renders when token info flow emits null`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 10.0).toGem(), mockChartDateValue(date = 61000L, value = 12.0).toGem(), mockChartDateValue(date = 121000L, value = 14.0).toGem()), baseValue = 10.0)
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.values?.size == chart.values.size }.chart.dataOrNull!!

        assertEquals(chart.values.size, uiModel.values.size)
        assertEquals(14.0, uiModel.header?.value?.value)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `current point overlay is skipped when local price info is missing`() = runTest(testDispatcher) {
        val chart =
            mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 100.0).toGem(), mockChartDateValue(date = 61000L, value = 105.0).toGem(), mockChartDateValue(date = 121000L, value = 110.0).toGem()), baseValue = 100.0)
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.values?.size == chart.values.size }.chart.dataOrNull!!

        assertEquals(chart.values.size, uiModel.values.size)
        assertEquals(110.0, uiModel.header?.value?.value)
    }

    @Test
    fun `initial request uses currency flow without waiting for session object`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 1.0).toGem(), mockChartDateValue(date = 61000L, value = 2.0).toGem()), baseValue = 1.0)
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.values?.size == chart.values.size }.chart.dataOrNull!!

        coVerify(exactly = 1) {
            chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem())
        }
        assertEquals(chart.values.size, uiModel.values.size)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `initial request uses saved chart period`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 1.0).toGem(), mockChartDateValue(date = 61000L, value = 2.0).toGem()), baseValue = 1.0)
        savedPeriod = ChartPeriod.Month
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Month.toGem()) } returns chart

        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart.dataOrNull?.values?.size == chart.values.size }

        assertEquals(ChartPeriod.Month, viewModel.chartUIState.value.period)
        coVerify(exactly = 1) {
            chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Month.toGem())
        }
    }

    @Test
    fun `selecting period stores chart period`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        viewModel.setPeriod(ChartPeriod.Month)
        val state = viewModel.chartUIState.first { it.period == ChartPeriod.Month }

        assertEquals(ChartPeriod.Month, state.period)
        verify(exactly = 1) { chartService.setChartPeriod(ChartPeriod.Month.toGem()) }
    }

    @Test
    fun `selecting a period loads the chart without the refresh indicator`() = runTest(testDispatcher) {
        val day = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 1.0).toGem(), mockChartDateValue(date = 61000L, value = 2.0).toGem()), baseValue = 1.0)
        val week = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 3.0).toGem(), mockChartDateValue(date = 61000L, value = 4.0).toGem(), mockChartDateValue(date = 121000L, value = 5.0).toGem()), baseValue = 3.0)
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns day

        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart is StateViewType.Data }
        backgroundScope.launch { viewModel.isRefreshing.collect {} }
        backgroundScope.launch { viewModel.chartUIState.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        val inFlight = CompletableDeferred<Unit>()
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Week.toGem()) } coAnswers {
            inFlight.await()
            week
        }
        viewModel.setPeriod(ChartPeriod.Week)
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(ChartPeriod.Week, viewModel.chartUIState.value.period)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Loading)
        assertEquals(false, viewModel.isRefreshing.value)

        inFlight.complete(Unit)
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(false, viewModel.isRefreshing.value)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `opening the chart loads it without the refresh indicator`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 1.0).toGem(), mockChartDateValue(date = 61000L, value = 2.0).toGem()), baseValue = 1.0)
        val inFlight = CompletableDeferred<Unit>()
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } coAnswers {
            inFlight.await()
            chart
        }

        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.isRefreshing.collect {} }
        backgroundScope.launch { viewModel.chartUIState.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Loading)
        assertEquals(false, viewModel.isRefreshing.value)

        inFlight.complete(Unit)
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
        assertEquals(false, viewModel.isRefreshing.value)
    }

    @Test
    fun `a pull to refresh keeps the chart that is already drawn`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 1.0).toGem(), mockChartDateValue(date = 61000L, value = 2.0).toGem(), mockChartDateValue(date = 121000L, value = 3.0).toGem()), baseValue = 1.0)
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart is StateViewType.Data }
        backgroundScope.launch { viewModel.isRefreshing.collect {} }
        backgroundScope.launch { viewModel.chartUIState.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        val inFlight = CompletableDeferred<Unit>()
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } coAnswers {
            inFlight.await()
            chart
        }
        viewModel.refresh()
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(true, viewModel.isRefreshing.value)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)

        inFlight.complete(Unit)
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(false, viewModel.isRefreshing.value)
        coVerify(exactly = 2) { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) }
    }

    @Test
    fun `a failed refresh leaves the loaded chart alone`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(mockChartDateValue(date = 1000L, value = 1.0).toGem(), mockChartDateValue(date = 61000L, value = 2.0).toGem(), mockChartDateValue(date = 121000L, value = 3.0).toGem()), baseValue = 1.0)
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart is StateViewType.Data }

        backgroundScope.launch { viewModel.chartUIState.collect {} }
        backgroundScope.launch { viewModel.isRefreshing.collect {} }
        testDispatcher.scheduler.advanceUntilIdle()

        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } throws GemServiceException.Api("offline")
        viewModel.refresh()
        testDispatcher.scheduler.advanceUntilIdle()

        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
        assertEquals(false, viewModel.isRefreshing.value)
    }

    private fun createViewModel(): ChartValuesViewModel = ChartValuesViewModel(
        getCurrentCurrency = getCurrentCurrency,
        priceQuery = mockk { every { this@mockk(asset.id) } returns flowOf(null) },
        chartService = chartService,
        assetId = asset.id,
        observeRefreshInterval = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
        context = mockk(relaxed = true),
    ).also(viewModels::add)
}
