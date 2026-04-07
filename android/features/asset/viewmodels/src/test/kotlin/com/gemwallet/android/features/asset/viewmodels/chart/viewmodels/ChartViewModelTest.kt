package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetAssetChartData
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockChartPrices
import com.wallet.core.primitives.ChartPeriod
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetSolanaUSDC()
    private val currencyFlow = MutableStateFlow(Currency.USD)
    private val viewModels = mutableListOf<ViewModel>()

    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }
    private val getAssetChartData = mockk<GetAssetChartData>(relaxed = true)

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        viewModels.forEach { it.viewModelScope.cancel() }
        viewModels.clear()
        Dispatchers.resetMain()
    }

    @Test
    fun `historical chart renders when token info flow emits null`() = runTest(testDispatcher) {
        val prices = mockChartPrices(values = listOf(10f, 12f, 14f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        every { assetsRepository.getTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD) } returns prices

        val viewModel = createViewModel(tokenInfoFlow)
        val uiModel = viewModel.chartUIModel.first { it.chartPoints.size == prices.size }

        assertEquals(prices.size, uiModel.chartPoints.size)
        assertNull(uiModel.currentPoint)
        assertFalse(viewModel.chartUIState.value.loading)
    }

    @Test
    fun `current point overlay is skipped when local price info is missing`() = runTest(testDispatcher) {
        val prices = mockChartPrices(values = listOf(100f, 105f, 110f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(mockAssetInfo(asset = asset))
        every { assetsRepository.getTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD) } returns prices

        val viewModel = createViewModel(tokenInfoFlow)
        val uiModel = viewModel.chartUIModel.first { it.chartPoints.size == prices.size }

        assertEquals(prices.size, uiModel.chartPoints.size)
        assertNull(uiModel.currentPoint)
    }

    @Test
    fun `initial request uses currency flow without waiting for session object`() = runTest(testDispatcher) {
        val prices = mockChartPrices(values = listOf(1f, 2f))
        val tokenInfoFlow = MutableStateFlow<AssetInfo?>(null)
        every { assetsRepository.getTokenInfo(asset.id) } returns tokenInfoFlow
        coEvery { getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD) } returns prices

        val viewModel = createViewModel(tokenInfoFlow)
        val uiModel = viewModel.chartUIModel.first { it.chartPoints.size == prices.size }

        coVerify(exactly = 1) {
            getAssetChartData.getAssetChartData(asset.id, ChartPeriod.Day, Currency.USD)
        }
        assertEquals(prices.size, uiModel.chartPoints.size)
        assertFalse(viewModel.chartUIState.value.loading)
    }

    private fun createViewModel(tokenInfoFlow: MutableStateFlow<AssetInfo?>): ChartViewModel {
        every { assetsRepository.getTokenInfo(asset.id) } returns tokenInfoFlow
        return ChartViewModel(
            assetsRepository = assetsRepository,
            sessionRepository = sessionRepository,
            getAssetChartData = getAssetChartData,
            assetId = asset.id,
        ).also(viewModels::add)
    }
}
