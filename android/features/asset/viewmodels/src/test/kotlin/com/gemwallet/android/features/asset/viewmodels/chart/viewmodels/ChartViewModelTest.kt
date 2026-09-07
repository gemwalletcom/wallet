package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.ext.toGem
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.GemChart
import uniffi.gemstone.GemChartService
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.ChartDateValue
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
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetSolanaUSDC()
    private val currencyFlow = MutableStateFlow(Currency.USD)
    private val viewModels = mutableListOf<ViewModel>()

    private val getAssetTokenInfo = mockk<GetAssetTokenInfo>(relaxed = true)
    private val getCurrentCurrency = mockk<GetCurrentCurrency>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }
    private val chartService = mockk<GemChartService>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { chartService.chartPeriod() } returns ChartPeriod.Day.toGem()
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
        val prices = mockChartPrices(values = listOf(10f, 12f, 14f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        every { getAssetTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns prices.toGemChart()

        val viewModel = createViewModel(tokenInfoFlow)
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == prices.size }.chart.dataOrNull!!

        assertEquals(prices.size, uiModel.chartPoints.size)
        assertNull(uiModel.currentPoint)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `current point overlay is skipped when local price info is missing`() = runTest(testDispatcher) {
        val prices = mockChartPrices(values = listOf(100f, 105f, 110f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(mockAssetInfo(asset = asset))
        every { getAssetTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns prices.toGemChart()

        val viewModel = createViewModel(tokenInfoFlow)
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == prices.size }.chart.dataOrNull!!

        assertEquals(prices.size, uiModel.chartPoints.size)
        assertNull(uiModel.currentPoint)
    }

    @Test
    fun `initial request uses currency flow without waiting for session object`() = runTest(testDispatcher) {
        val prices = mockChartPrices(values = listOf(1f, 2f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        every { getAssetTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem()) } returns prices.toGemChart()

        val viewModel = createViewModel(tokenInfoFlow)
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == prices.size }.chart.dataOrNull!!

        coVerify(exactly = 1) {
            chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Day.toGem())
        }
        assertEquals(prices.size, uiModel.chartPoints.size)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `initial request uses saved chart period`() = runTest(testDispatcher) {
        val prices = mockChartPrices(values = listOf(1f, 2f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        every { chartService.chartPeriod() } returns ChartPeriod.Month.toGem()
        every { getAssetTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Month.toGem()) } returns prices.toGemChart()

        val viewModel = createViewModel(tokenInfoFlow)
        viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == prices.size }

        assertEquals(ChartPeriod.Month, viewModel.chartUIState.value.period)
        coVerify(exactly = 1) {
            chartService.syncCharts(asset.id.toIdentifier(), ChartPeriod.Month.toGem())
        }
    }

    @Test
    fun `selecting period stores chart period`() = runTest(testDispatcher) {
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        val viewModel = createViewModel(tokenInfoFlow)

        viewModel.setPeriod(ChartPeriod.Month)
        val state = viewModel.chartUIState.first { it.period == ChartPeriod.Month }

        assertEquals(ChartPeriod.Month, state.period)
        verify(exactly = 1) { chartService.setChartPeriod(ChartPeriod.Month.toGem()) }
    }

    private fun createViewModel(tokenInfoFlow: MutableStateFlow<AssetInfo?>): ChartViewModel {
        every { getAssetTokenInfo(asset.id) } returns tokenInfoFlow
        return ChartViewModel(
            getAssetTokenInfo = getAssetTokenInfo,
            getCurrentCurrency = getCurrentCurrency,
            chartService = chartService,
            assetId = asset.id,
        ).also(viewModels::add)
    }

    private fun mockChartPrices(values: List<Float>): List<ChartDateValue> =
        values.mapIndexed { index, value -> ChartDateValue(date = 1_000L + index * 60_000L, value = value.toDouble()) }

    private fun List<ChartDateValue>.toGemChart() = GemChart(values = map { it.toGem() }, current = null)
}
