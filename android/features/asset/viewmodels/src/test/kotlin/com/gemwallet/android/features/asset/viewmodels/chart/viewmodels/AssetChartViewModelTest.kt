package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetMarket
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.model.AssetInfo
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class AssetChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetSolanaUSDC()
    private val assetInfoFlow = MutableStateFlow<AssetInfo?>(mockAssetInfo(asset = asset))
    private val linksFlow = MutableStateFlow<List<AssetLink>>(emptyList())
    private val marketFlow = MutableStateFlow<AssetMarket?>(null)
    private val currencyFlow = MutableStateFlow(Currency.USD)

    private val assetsRepository = mockk<AssetsRepository>(relaxed = true) {
        every { getTokenInfo(asset.id) } returns assetInfoFlow
        every { getAssetLinks(asset.id) } returns linksFlow
        every { getAssetMarket(asset.id) } returns marketFlow
    }
    private val getCurrentBlockExplorer = mockk<GetCurrentBlockExplorer>(relaxed = true) {
        every { getCurrentBlockExplorer(asset.id.chain) } returns "Solscan"
    }
    private val getPriceAlerts = mockk<GetPriceAlerts>(relaxed = true) {
        every { getPriceAlerts(asset.id) } returns MutableStateFlow(emptyList())
    }
    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `local asset info bootstraps ui without waiting for market or links`() = runTest(testDispatcher) {
        val viewModel = AssetChartViewModel(
            assetsRepository = assetsRepository,
            getCurrentBlockExplorer = getCurrentBlockExplorer,
            getPriceAlerts = getPriceAlerts,
            sessionRepository = sessionRepository,
            assetId = asset.id,
        )
        val uiModel = viewModel.marketUIModel.first { it != null }!!

        assertEquals(asset, uiModel.asset)
        assertEquals(asset.name, uiModel.assetTitle)
        assertEquals(0, uiModel.assetLinks.size)
        assertEquals(Currency.USD, uiModel.currency)
        assertNull(uiModel.marketInfo)
        assertEquals("Solscan", uiModel.explorerName)
    }

    @Test
    fun `repo updates populate market and links without changing bootstrap asset`() = runTest(testDispatcher) {
        val viewModel = AssetChartViewModel(
            assetsRepository = assetsRepository,
            getCurrentBlockExplorer = getCurrentBlockExplorer,
            getPriceAlerts = getPriceAlerts,
            sessionRepository = sessionRepository,
            assetId = asset.id,
        )
        advanceUntilIdle()

        val market = mockAssetMarket(marketCap = 1234.0)
        linksFlow.value = listOf(mockAssetLink())
        marketFlow.value = market
        currencyFlow.value = Currency.EUR

        val uiModel = viewModel.marketUIModel.first {
            it?.assetLinks?.isNotEmpty() == true && it.marketInfo == market && it.currency == Currency.EUR
        }!!

        assertEquals(asset, uiModel.asset)
        assertEquals(asset.name, uiModel.assetTitle)
        assertEquals(1, uiModel.assetLinks.size)
        assertEquals(market, uiModel.marketInfo)
        assertEquals(Currency.EUR, uiModel.currency)
    }
}
