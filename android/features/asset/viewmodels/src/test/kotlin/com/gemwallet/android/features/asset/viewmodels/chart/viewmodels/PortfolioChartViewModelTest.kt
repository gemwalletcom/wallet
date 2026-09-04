package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.Session
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.data.services.gemstone.perpetual.ObservePerpetualWallet
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.ChartDateValue
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.PortfolioChartData
import com.wallet.core.primitives.PortfolioChartType
import com.wallet.core.primitives.PortfolioData
import com.wallet.core.primitives.PortfolioStatistic
import com.wallet.core.primitives.PortfolioType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
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
import java.util.concurrent.TimeUnit
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class PortfolioChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val session = mockSession()
    private val sessionFlow = MutableStateFlow<Session?>(session)
    private val viewModels = mutableListOf<ViewModel>()

    private val getSession = mockk<GetSession> {
        every { this@mockk.invoke() } returns sessionFlow
    }
    private val observePerpetualWallet = mockk<ObservePerpetualWallet>(relaxed = true)
    private val service = mockk<uniffi.gemstone.GemPortfolioServiceInterface>()

    private fun stubPortfolio(type: PortfolioType? = null, period: ChartPeriod? = null, data: PortfolioData) {
        coEvery {
            service.portfolioData(any(), type?.toGem() ?: any(), period?.toJson() ?: any())
        } returns data.toJson()
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { observePerpetualWallet() } returns flowOf(null)
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
    fun `renders chart when portfolio has values`() = runTest(testDispatcher) {
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, portfolioData(listOf(10f, 12f, 14f)))

        val viewModel = createViewModel()
        val state = viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == 3 }

        assertEquals(3, state.chart.dataOrNull?.chartPoints?.size)
    }

    @Test
    fun `initial request uses all period by default`() = runTest(testDispatcher) {
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, portfolioData(listOf(1f, 2f)))

        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == 2 }

        assertEquals(ChartPeriod.All, viewModel.chartUIState.first { it.chart != StateViewType.Loading }.period)
        coVerify(exactly = 1) { service.portfolioData(session.wallet.toJson(), PortfolioType.Wallet.toGem(), ChartPeriod.All.toJson()) }
    }

    @Test
    fun `selecting period updates state and refetches`() = runTest(testDispatcher) {
        stubPortfolio(data = portfolioData(listOf(1f, 2f)))
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.Month, portfolioData(listOf(1f, 2f, 3f)))
        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        viewModel.setPeriod(ChartPeriod.Month)
        val state = viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == 3 }

        assertEquals(ChartPeriod.Month, state.period)
        coVerify { service.portfolioData(any(), PortfolioType.Wallet.toGem(), ChartPeriod.Month.toJson()) }
    }

    @Test
    fun `resets period when the selected period is unavailable`() = runTest(testDispatcher) {
        val periods = listOf(ChartPeriod.Day, ChartPeriod.Week, ChartPeriod.Month)
        stubPortfolio(period = ChartPeriod.All, data = portfolioData(listOf(1f, 2f), availablePeriods = periods))
        stubPortfolio(period = ChartPeriod.Day, data = portfolioData(listOf(1f, 2f, 3f), availablePeriods = periods))
        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        val state = viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == 3 }

        assertEquals(ChartPeriod.Day, state.period)
        coVerify { service.portfolioData(any(), PortfolioType.Wallet.toGem(), ChartPeriod.Day.toJson()) }
    }

    @Test
    fun `starts on perpetuals when opened with perpetuals type`() = runTest(testDispatcher) {
        stubPortfolio(PortfolioType.Perpetuals, ChartPeriod.All, portfolioData(listOf(1f, 2f)))
        val viewModel = createViewModel(initialType = PortfolioType.Perpetuals)
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        viewModel.chartUIState.first { it.chart.dataOrNull?.chartPoints?.size == 2 }

        assertEquals(PortfolioType.Perpetuals, viewModel.selectedType.value)
        coVerify(exactly = 0) { service.portfolioData(any(), PortfolioType.Wallet.toGem(), any()) }
    }

    @Test
    fun `shows error state when the portfolio request fails`() = runTest(testDispatcher) {
        coEvery { service.portfolioData(any(), any(), any()) } throws IllegalStateException("network down")
        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        val state = viewModel.chartUIState.first { it.chart != StateViewType.Loading }

        assertEquals(StateViewType.Error, state.chart)
    }

    @Test
    fun `flat chart without variation is empty`() = runTest(testDispatcher) {
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, portfolioData(listOf(5f, 5f, 5f)))
        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        val state = viewModel.chartUIState.first { it.chart != StateViewType.Loading }

        assertEquals(StateViewType.NoData, state.chart)
    }

    @Test
    fun `exposes all time statistics from portfolio`() = runTest(testDispatcher) {
        val allTimeHigh = ChartValuePercentage(date = 1L, value = 99f, percentage = 5f)
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, portfolioData(listOf(1f, 2f), statistics = listOf(PortfolioStatistic.AllTimeHigh(allTimeHigh))))

        val viewModel = createViewModel()
        val statistics = viewModel.statistics.first { it.isNotEmpty() }

        assertEquals(listOf(PortfolioStatistic.AllTimeHigh(allTimeHigh)), statistics)
    }

    private fun portfolioData(
        values: List<Float>,
        statistics: List<PortfolioStatistic> = emptyList(),
        availablePeriods: List<ChartPeriod> = emptyList(),
    ) = PortfolioData(
        charts = listOf(
            PortfolioChartData(
                chartType = PortfolioChartType.Value,
                values = values.mapIndexed { index, value ->
                    ChartDateValue(date = TimeUnit.SECONDS.toMillis((index + 1).toLong()), value = value.toDouble())
                },
            ),
        ),
        statistics = statistics,
        availablePeriods = availablePeriods,
    )

    private fun createViewModel(initialType: PortfolioType = PortfolioType.Wallet) = PortfolioChartViewModel(
        service = service,
        getSession = getSession,
        observePerpetualWallet = observePerpetualWallet,
        initialType = initialType,
    ).also(viewModels::add)
}
