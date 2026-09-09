package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import uniffi.gemstone.GemChartServiceInterface
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetById
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModelFactory
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetMarket
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
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
import uniffi.gemstone.GemAssetMarketRow
import uniffi.gemstone.GemAssetMarketRows

@OptIn(ExperimentalCoroutinesApi::class)
class AssetChartViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetSolanaUSDC()
    private val viewModels = mutableListOf<ViewModel>()
    private val assetFlow = MutableStateFlow(asset)
    private val linksFlow = MutableStateFlow<List<AssetLink>>(emptyList())
    private val marketFlow = MutableStateFlow<AssetMarket?>(null)
    private val currencyFlow = MutableStateFlow(Currency.USD)

    private val getAssetById = mockk<GetAssetById>(relaxed = true)
    private val getAssetLinks = mockk<GetAssetLinks>(relaxed = true)
    private val getAssetMarket = mockk<GetAssetMarket>(relaxed = true)
    private val walletAssetsFlow = MutableStateFlow<List<AssetInfo>>(emptyList())
    private val getWalletAssets = mockk<GetWalletAssets>(relaxed = true) {
        every { this@mockk.invoke() } returns walletAssetsFlow
    }
    private val chartService = mockk<GemChartServiceInterface>(relaxed = true)
    private val getPriceAlerts = mockk<GetPriceAlerts>(relaxed = true)
    private val getCurrentCurrency = mockk<GetCurrentCurrency>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }
    private val emptyRows = GemAssetMarketRows(market = emptyList(), contract = emptyList(), supply = emptyList(), allTime = emptyList())

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getAssetById(asset.id) } returns assetFlow
        every { getAssetLinks(asset.id) } returns linksFlow
        every { getAssetMarket(asset.id) } returns marketFlow
        every { getPriceAlerts(asset.id) } returns MutableStateFlow<List<PriceAlertDataAggregate>>(emptyList())
        every { chartService.marketRows(any(), any()) } returns emptyRows
    }

    @After
    fun tearDown() {
        viewModels.forEach { it.viewModelScope.cancel() }
        viewModels.clear()
        Dispatchers.resetMain()
    }

    @Test
    fun `a stored asset gives the scene its rows and title before any flow emits`() = runTest(testDispatcher) {
        walletAssetsFlow.value = listOf(mockAssetInfo(asset))
        every { chartService.marketRows(asset.toGem(), null) } returns emptyRows.copy(
            contract = listOf(GemAssetMarketRow.Contract(tokenId = requireNotNull(asset.id.tokenId), explorer = null)),
        )

        val viewModel = createViewModel()

        val uiModel = requireNotNull(viewModel.marketUIModel.value)
        assertEquals(asset.chain, uiModel.chain)
        assertEquals(1, uiModel.contractRows.size)
        assertEquals(asset.name, viewModel.title.value)
    }

    @Test
    fun `an asset the wallet does not hold leaves the scene empty until it loads`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        assertNull(viewModel.marketUIModel.value)
        assertEquals("", viewModel.title.value)
        assertEquals(asset.name, viewModel.title.first { it.isNotBlank() })
    }

    @Test
    fun `repo updates populate market and links without changing bootstrap asset`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        advanceUntilIdle()

        val market = mockAssetMarket(marketCap = 1234.0)
        every { chartService.marketRows(asset.toGem(), market.toGem()) } returns emptyRows.copy(
            market = listOf(GemAssetMarketRow.MarketCap(value = 1234.0, rank = null)),
        )
        linksFlow.value = listOf(mockAssetLink())
        marketFlow.value = market
        currencyFlow.value = Currency.EUR

        val uiModel = viewModel.marketUIModel.first {
            it?.links?.isNotEmpty() == true && it.marketRows.isNotEmpty() && it.currency == Currency.EUR
        }!!

        assertEquals(asset.chain, uiModel.chain)
        assertEquals(1, uiModel.links.size)
        assertEquals(1, uiModel.marketRows.size)
        assertEquals(Currency.EUR, uiModel.currency)
    }

    private fun createViewModel(): AssetChartViewModel = AssetChartViewModel(
        getAssetById = getAssetById,
        getAssetLinks = getAssetLinks,
        getAssetMarket = getAssetMarket,
        getWalletAssets = getWalletAssets,
        chartService = chartService,
        getPriceAlerts = getPriceAlerts,
        getCurrentCurrency = getCurrentCurrency,
        marketUIModelFactory = AssetMarketUIModelFactory(),
        assetId = asset.id,
    ).also(viewModels::add)
}
