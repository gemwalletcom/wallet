package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockPortfolioChartData
import com.gemwallet.android.testkit.mockPortfolioData
import com.gemwallet.android.testkit.mockSession
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.PortfolioType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.job
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.GemChartPhase
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPortfolioResult
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.PortfolioChartType
import uniffi.gemstone.PortfolioData
import uniffi.gemstone.PortfolioStatistic
import uniffi.gemstone.portfolioSession
import java.util.concurrent.TimeUnit

@OptIn(ExperimentalCoroutinesApi::class)
class PortfolioViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(testDispatcher)

    private val session = mockSession()
    private val sessionFlow = MutableStateFlow<Session?>(session)
    private val viewModels = mutableListOf<ViewModel>()

    private val getSession = mockk<GetSession> {
        every { this@mockk.invoke() } returns sessionFlow
    }
    private val service = mockk<uniffi.gemstone.GemPortfolioServiceInterface>()
    private val allPeriods = portfolioSession(PortfolioType.Wallet.toGem()).viewState().periods

    private fun stubPortfolio(type: PortfolioType, period: ChartPeriod, data: PortfolioData) {
        coEvery {
            service.refresh(any(), match { request -> request.portfolioType == type.toGem() && request.period == period.toGem() })
        } answers { GemPortfolioResult(request = secondArg(), state = GemLoadState.Data, data = data) }
    }

    @Before
    fun setUp() {
        every { service.showPerpetuals(any(), any()) } returns false
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
    fun `renders chart when portfolio has values`() = runTest(testDispatcher) {
        val values = listOf(10.0, 12.0, 14.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, mockPortfolioData(charts = listOf(mockPortfolioChartData(values = values)), availablePeriods = allPeriods))

        val viewModel = createViewModel()
        advanceUntilIdle()

        assertEquals(3, (viewModel.chartUIState.value.phase as? GemChartPhase.Data)?.data?.values?.size)
    }

    @Test
    fun `initial request uses all period by default`() = runTest(testDispatcher) {
        val values = listOf(1.0, 2.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, mockPortfolioData(charts = listOf(mockPortfolioChartData(values = values)), availablePeriods = allPeriods))

        val viewModel = createViewModel()
        advanceUntilIdle()

        assertEquals(ChartPeriod.All, viewModel.chartUIState.value.period)
        coVerify(exactly = 1) {
            service.refresh(session.wallet.toGem(), match { it.portfolioType == PortfolioType.Wallet.toGem() && it.period == ChartPeriod.All.toGem() })
        }
    }

    @Test
    fun `selecting period updates state and refetches`() = runTest(testDispatcher) {
        val allValues = listOf(1.0, 2.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        val monthValues = listOf(1.0, 2.0, 3.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, mockPortfolioData(charts = listOf(mockPortfolioChartData(values = allValues)), availablePeriods = allPeriods))
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.Month, mockPortfolioData(charts = listOf(mockPortfolioChartData(values = monthValues)), availablePeriods = allPeriods))
        val viewModel = createViewModel()
        advanceUntilIdle()
        assertEquals(2, (viewModel.chartUIState.value.phase as? GemChartPhase.Data)?.data?.values?.size)

        viewModel.setPeriod(ChartPeriod.Month)
        advanceUntilIdle()

        val state = viewModel.chartUIState.value
        assertEquals(ChartPeriod.Month, state.period)
        assertEquals(3, (state.phase as? GemChartPhase.Data)?.data?.values?.size)
        coVerifyOrder {
            service.refresh(any(), match { it.period == ChartPeriod.All.toGem() })
            service.refresh(any(), match { it.period == ChartPeriod.Month.toGem() })
        }
        coVerify(exactly = 2) { service.refresh(any(), any()) }
    }

    @Test
    fun `resets period when the selected period is unavailable`() = runTest(testDispatcher) {
        val periods = listOf(ChartPeriod.Day, ChartPeriod.Week, ChartPeriod.Month).map { it.toGem() }
        val values = listOf(1.0, 2.0, 3.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        stubPortfolio(
            PortfolioType.Perpetuals,
            ChartPeriod.All,
            mockPortfolioData(charts = listOf(mockPortfolioChartData(chartType = PortfolioChartType.PNL), mockPortfolioChartData(chartType = PortfolioChartType.VALUE)), availablePeriods = periods),
        )
        stubPortfolio(
            PortfolioType.Perpetuals,
            ChartPeriod.Day,
            mockPortfolioData(charts = listOf(mockPortfolioChartData(chartType = PortfolioChartType.PNL, values = values), mockPortfolioChartData(chartType = PortfolioChartType.VALUE, values = values)), availablePeriods = periods),
        )

        val viewModel = createViewModel(initialType = PortfolioType.Perpetuals)
        advanceUntilIdle()

        val state = viewModel.chartUIState.value
        assertEquals(ChartPeriod.Day, state.period)
        assertEquals(3, (state.phase as? GemChartPhase.Data)?.data?.values?.size)
        coVerifyOrder {
            service.refresh(any(), match { it.portfolioType == PortfolioType.Perpetuals.toGem() && it.period == ChartPeriod.All.toGem() })
            service.refresh(any(), match { it.portfolioType == PortfolioType.Perpetuals.toGem() && it.period == ChartPeriod.Day.toGem() })
        }
    }

    @Test
    fun `starts on perpetuals when opened with perpetuals type`() = runTest(testDispatcher) {
        val values = listOf(1.0, 2.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        stubPortfolio(
            PortfolioType.Perpetuals,
            ChartPeriod.All,
            mockPortfolioData(charts = listOf(mockPortfolioChartData(chartType = PortfolioChartType.PNL, values = values), mockPortfolioChartData(chartType = PortfolioChartType.VALUE, values = values)), availablePeriods = allPeriods),
        )

        val viewModel = createViewModel(initialType = PortfolioType.Perpetuals)
        advanceUntilIdle()

        assertEquals(PortfolioType.Perpetuals, viewModel.selectedType.value)
        assertEquals(2, (viewModel.chartUIState.value.phase as? GemChartPhase.Data)?.data?.values?.size)
        coVerify(exactly = 0) { service.refresh(any(), match { it.portfolioType == PortfolioType.Wallet.toGem() }) }
    }

    @Test
    fun `a failed portfolio request shows no data and only being offline shows an error`() = runTest(testDispatcher) {
        coEvery { service.refresh(any(), any()) } answers {
            GemPortfolioResult(request = secondArg(), state = GemLoadState.Error(GemServiceException.Gateway("network down")), data = null)
        }
        val failed = createViewModel()
        advanceUntilIdle()

        assertEquals(GemChartPhase.NoData, failed.chartUIState.value.phase)

        coEvery { service.refresh(any(), any()) } answers {
            GemPortfolioResult(request = secondArg(), state = GemLoadState.Error(GemServiceException.Offline()), data = null)
        }
        val offline = createViewModel()
        advanceUntilIdle()

        assertTrue(offline.chartUIState.value.phase is GemChartPhase.Failed)
    }

    @Test
    fun `flat chart without variation is empty`() = runTest(testDispatcher) {
        val values = listOf(5.0, 5.0, 5.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        stubPortfolio(PortfolioType.Wallet, ChartPeriod.All, mockPortfolioData(charts = listOf(mockPortfolioChartData(values = values)), availablePeriods = allPeriods))

        val viewModel = createViewModel()
        advanceUntilIdle()

        assertEquals(GemChartPhase.NoData, viewModel.chartUIState.value.phase)
    }

    @Test
    fun `exposes all time statistics from portfolio`() = runTest(testDispatcher) {
        val values = listOf(1.0, 2.0).mapIndexed { index, value -> ChartDateValue(date = TimeUnit.SECONDS.toMillis(index + 1L), value = value) }
        val allTimeHigh = ChartValuePercentage(date = 1L, value = 99f, percentage = 5f).toGem()
        stubPortfolio(
            PortfolioType.Wallet,
            ChartPeriod.All,
            mockPortfolioData(charts = listOf(mockPortfolioChartData(values = values)), statistics = listOf(PortfolioStatistic.AllTimeHigh(allTimeHigh)), availablePeriods = allPeriods),
        )

        val viewModel = createViewModel()
        advanceUntilIdle()

        val row = viewModel.statistics.value.single()
        assertTrue(row is GemListRow.AllTime && row.title == GemListRowTitle.ALL_TIME_HIGH)
        assertEquals(5.0, (row as GemListRow.AllTime).change.value, 0.0)
        assertEquals(99.0, row.value.value, 0.0)
    }

    private fun createViewModel(initialType: PortfolioType = PortfolioType.Wallet) = PortfolioViewModel(
        service = service,
        getSession = getSession,
        initialType = initialType,
        observeRefreshInterval = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
    ).also(viewModels::add)
}
