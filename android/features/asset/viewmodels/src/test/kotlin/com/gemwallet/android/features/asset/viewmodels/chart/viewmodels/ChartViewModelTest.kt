package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetAssetChartData
import com.gemwallet.android.application.assets.coordinators.GetAssetTokenInfo
import com.gemwallet.android.application.assets.coordinators.GetChartPeriod
import com.gemwallet.android.application.assets.coordinators.SetChartPeriod
import com.gemwallet.android.application.session.coordinators.GetCurrentCurrency
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockChartPrices
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
    private val getAssetChartData = mockk<GetAssetChartData>(relaxed = true)
    private val getChartPeriod = mockk<GetChartPeriod>()
    private val setChartPeriod = mockk<SetChartPeriod>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getChartPeriod() } returns ChartPeriod.Day
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
        coEvery { getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD) } returns prices

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
        coEvery { getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD) } returns prices

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
        coEvery { getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD) } returns prices

        val viewModel = createViewModel(tokenInfoFlow)
        val uiModel = viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == prices.size }.chart.dataOrNull!!

        coVerify(exactly = 1) {
            getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD)
        }
        assertEquals(prices.size, uiModel.chartPoints.size)
        assertEquals(true, viewModel.chartUIState.value.chart is StateViewType.Data)
    }

    @Test
    fun `initial request uses saved chart period`() = runTest(testDispatcher) {
        val prices = mockChartPrices(values = listOf(1f, 2f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        every { getChartPeriod() } returns ChartPeriod.Month
        every { getAssetTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Month, Currency.USD) } returns prices

        val viewModel = createViewModel(tokenInfoFlow)
        viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == prices.size }

        assertEquals(ChartPeriod.Month, viewModel.chartUIState.value.period)
        coVerify(exactly = 1) {
            getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Month, Currency.USD)
        }
    }

    @Test
    fun `selecting period stores chart period`() = runTest(testDispatcher) {
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        val viewModel = createViewModel(tokenInfoFlow)

        viewModel.setPeriod(ChartPeriod.Month)
        val state = viewModel.chartUIState.first { it.period == ChartPeriod.Month }

        assertEquals(ChartPeriod.Month, state.period)
        verify(exactly = 1) { setChartPeriod(ChartPeriod.Month) }
    }

    private fun createViewModel(tokenInfoFlow: MutableStateFlow<AssetInfo?>): ChartViewModel {
        every { getAssetTokenInfo(asset.id) } returns tokenInfoFlow
        return ChartViewModel(
            getAssetTokenInfo = getAssetTokenInfo,
            getCurrentCurrency = getCurrentCurrency,
            getAssetChartData = getAssetChartData,
            getChartPeriod = getChartPeriod,
            setChartPeriod = setChartPeriod,
            assetId = asset.id,
        ).also(viewModels::add)
    }
}
