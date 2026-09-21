package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockGemChart
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.job
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

@OptIn(ExperimentalCoroutinesApi::class)
class ChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetSolanaUSDC()
    private val currencyFlow = MutableStateFlow(Currency.USD)
    private val viewModels = mutableListOf<ViewModel>()

    private val getCurrentCurrency = mockk<GetCurrentCurrency>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }
    private val chartService = mockk<GemChartService>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { chartService.chartPeriod() } returns ChartPeriod.Day.toGem()
        every { chartService.newSession() } answers {
            GemChartSession(chartService.chartPeriod(), currencyFlow.value.toGem(), chart = null, error = null, isLoading = true, isRefreshing = false)
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
        val chart = mockGemChart(values = listOf(10f, 12f, 14f))
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == chart.values.size }.chart.dataOrNull!!

        assertEquals(chart.values.size, uiModel.chart.values.size)
        assertEquals(14.0, uiModel.chart.header?.value?.value)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `current point overlay is skipped when local price info is missing`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(100f, 105f, 110f))
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == chart.values.size }.chart.dataOrNull!!

        assertEquals(chart.values.size, uiModel.chart.values.size)
        assertEquals(110.0, uiModel.chart.header?.value?.value)
    }

    @Test
    fun `initial request uses currency flow without waiting for session object`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(1f, 2f))
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns chart

        val viewModel = createViewModel()
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == chart.values.size }.chart.dataOrNull!!

        coVerify(exactly = 1) {
            chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem())
        }
        assertEquals(chart.values.size, uiModel.chart.values.size)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `initial request uses saved chart period`() = runTest(testDispatcher) {
        val chart = mockGemChart(values = listOf(1f, 2f))
        every { chartService.chartPeriod() } returns ChartPeriod.Month.toGem()
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Month.toGem()) } returns chart

        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == chart.values.size }

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

    private fun createViewModel(): ChartViewModel = ChartViewModel(
        getCurrentCurrency = getCurrentCurrency,
        chartService = chartService,
        assetId = asset.id,
        connectionStatusObserver = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
    ).also(viewModels::add)
}
