package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockPortfolioChartData
import com.gemwallet.android.testkit.mockPortfolioData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.dataOrNull
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.ChartValuePercentage
import com.wallet.core.primitives.Currency
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
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.ChartDateValue
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPortfolioResult
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.PortfolioChartType
import uniffi.gemstone.PortfolioData
import uniffi.gemstone.PortfolioStatistic
import java.util.concurrent.TimeUnit

@OptIn(ExperimentalCoroutinesApi::class)
class PortfolioChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val session = mockSession()
    private val sessionFlow = MutableStateFlow<Session?>(session)
    private val viewModels = mutableListOf<ViewModel>()

    private val getSession = mockk<GetSession> {
        every { this@mockk.invoke() } returns sessionFlow
    }
    private val service = mockk<uniffi.gemstone.GemPortfolioServiceInterface>()

    private fun stubPortfolio(type: PortfolioType? = null, period: ChartPeriod? = null, data: PortfolioData) {
        coEvery {
            service.refresh(
                any(),
                match { request ->
                    (type == null || request.portfolioType == type.toGem()) && (period == null || request.period == period.toGem())
                },
            )
        } answers { GemPortfolioResult(request = secondArg(), state = GemLoadState.Data, data = data) }
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
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
        Dispatchers.resetMain()
    }

    @Test
    fun `renders chart when portfolio has values`() = runTest(testDispatcher) {
        stubPortfolio(
            PortfolioType.Wallet,
            ChartPeriod.All,
            mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(10f, 12f, 14f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
            ),
        )

        val viewModel = createViewModel()
        val state = viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == 3 }

        assertEquals(3, state.chart.dataOrNull?.chart?.values?.size)
    }

    @Test
    fun `initial request uses all period by default`() = runTest(testDispatcher) {
        stubPortfolio(
            PortfolioType.Wallet,
            ChartPeriod.All,
            mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(1f, 2f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
            ),
        )

        val viewModel = createViewModel()
        viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == 2 }

        assertEquals(ChartPeriod.All, viewModel.chartUIState.first { it.chart != StateViewType.Loading }.period)
        coVerify(exactly = 1) {
            service.refresh(session.wallet.toGem(), match { it.portfolioType == PortfolioType.Wallet.toGem() && it.period == ChartPeriod.All.toGem() })
        }
    }

    @Test
    fun `selecting period updates state and refetches`() = runTest(testDispatcher) {
        stubPortfolio(
            data = mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(1f, 2f).mapIndexed { index, value ->
                            ChartDateValue(date = TimeUnit.SECONDS.toMillis((index + 1).toLong()), value = value.toDouble())
                        },
                    ),
                ),
            ),
        )
        stubPortfolio(
            PortfolioType.Wallet,
            ChartPeriod.Month,
            mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(1f, 2f, 3f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
            ),
        )
        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        viewModel.setPeriod(ChartPeriod.Month)
        val state = viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == 3 }

        assertEquals(ChartPeriod.Month, state.period)
        coVerify { service.refresh(any(), match { it.portfolioType == PortfolioType.Wallet.toGem() && it.period == ChartPeriod.Month.toGem() }) }
    }

    @Test
    fun `resets period when the selected period is unavailable`() = runTest(testDispatcher) {
        val periods = listOf(ChartPeriod.Day, ChartPeriod.Week, ChartPeriod.Month)
        stubPortfolio(
            period = ChartPeriod.All,
            data = mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(1f, 2f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
                availablePeriods = periods.map { it.toGem() },
            ),
        )
        stubPortfolio(
            period = ChartPeriod.Day,
            data = mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(1f, 2f, 3f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
                availablePeriods = periods.map { it.toGem() },
            ),
        )
        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        val state = viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == 3 }

        assertEquals(ChartPeriod.Day, state.period)
        coVerify { service.refresh(any(), match { it.portfolioType == PortfolioType.Wallet.toGem() && it.period == ChartPeriod.Day.toGem() }) }
    }

    @Test
    fun `starts on perpetuals when opened with perpetuals type`() = runTest(testDispatcher) {
        stubPortfolio(
            PortfolioType.Perpetuals,
            ChartPeriod.All,
            mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(1f, 2f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
            ),
        )
        val viewModel = createViewModel(initialType = PortfolioType.Perpetuals)
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        viewModel.chartUIState.first { it.chart.dataOrNull?.chart?.values?.size == 2 }

        assertEquals(PortfolioType.Perpetuals, viewModel.selectedType.value)
        coVerify(exactly = 0) { service.refresh(any(), match { it.portfolioType == PortfolioType.Wallet.toGem() }) }
    }

    @Test
    fun `a failed portfolio request shows no data and only being offline shows an error`() = runTest(testDispatcher) {
        coEvery { service.refresh(any(), any()) } answers {
            GemPortfolioResult(request = secondArg(), state = GemLoadState.Error(GemServiceException.Gateway("network down")), data = null)
        }
        val failed = createViewModel()
        backgroundScope.launch { failed.chartUIState.collect {} }

        assertEquals(StateViewType.NoData, failed.chartUIState.first { it.chart != StateViewType.Loading }.chart)

        coEvery { service.refresh(any(), any()) } answers {
            GemPortfolioResult(request = secondArg(), state = GemLoadState.Error(GemServiceException.Offline()), data = null)
        }
        val offline = createViewModel()
        backgroundScope.launch { offline.chartUIState.collect {} }

        assertTrue(offline.chartUIState.first { it.chart != StateViewType.Loading }.chart is StateViewType.Error)
    }

    @Test
    fun `flat chart without variation is empty`() = runTest(testDispatcher) {
        stubPortfolio(
            PortfolioType.Wallet,
            ChartPeriod.All,
            mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(5f, 5f, 5f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
            ),
        )
        val viewModel = createViewModel()
        backgroundScope.launch { viewModel.chartUIState.collect {} }

        val state = viewModel.chartUIState.first { it.chart != StateViewType.Loading }

        assertEquals(StateViewType.NoData, state.chart)
    }

    @Test
    fun `exposes all time statistics from portfolio`() = runTest(testDispatcher) {
        val allTimeHigh = ChartValuePercentage(date = 1L, value = 99f, percentage = 5f).toGem()
        stubPortfolio(
            PortfolioType.Wallet,
            ChartPeriod.All,
            mockPortfolioData(
                charts = listOf(
                    mockPortfolioChartData(
                        chartType = PortfolioChartType.VALUE,
                        values = listOf(1f, 2f).mapIndexed { index, value ->
                            ChartDateValue(
                                date = TimeUnit.SECONDS.toMillis(
                                    (
                                        index +
                                            1
                                        ).toLong(),
                                ),
                                value = value.toDouble(),
                            )
                        },
                    ),
                ),
                statistics = listOf(PortfolioStatistic.AllTimeHigh(allTimeHigh)),
            ),
        )

        val viewModel = createViewModel()
        val statistics = viewModel.statistics.first { it.isNotEmpty() }

        val row = statistics.single()
        assertTrue(row is GemListRow.AllTime && row.title == GemListRowTitle.ALL_TIME_HIGH)
        assertEquals(5.0, (row as GemListRow.AllTime).change.value, 0.0)
        assertEquals(99.0, row.value.value, 0.0)
    }

    private fun createViewModel(initialType: PortfolioType = PortfolioType.Wallet) = PortfolioChartViewModel(
        service = service,
        getSession = getSession,
        initialType = initialType,
        observeRefreshInterval = mockk(relaxed = true),
        ioDispatcher = testDispatcher,
        context = mockk(relaxed = true),
    ).also(viewModels::add)
}
