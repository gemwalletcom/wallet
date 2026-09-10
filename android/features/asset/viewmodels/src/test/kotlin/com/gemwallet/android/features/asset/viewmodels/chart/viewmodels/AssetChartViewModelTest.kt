package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.pricealerts.aggregates.PriceAlertDataAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.asset.viewmodels.chart.models.AssetMarketUIModelFactory
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetMarket
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockPriceAlert
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.Currency
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
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
import uniffi.gemstone.GemChartSection
import uniffi.gemstone.GemChartServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class AssetChartViewModelTest {
    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetSolanaUSDC()
    private val viewModels = mutableListOf<ViewModel>()

    private val assetInfoFlow = MutableStateFlow<AssetInfo?>(mockAssetInfo(asset))
    private val linksFlow = MutableStateFlow<List<AssetLink>>(emptyList())
    private val marketFlow = MutableStateFlow<AssetMarket?>(null)
    private val currencyFlow = MutableStateFlow(Currency.USD)

    private val getAssetTokenInfo = mockk<GetAssetTokenInfo>(relaxed = true)
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

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getAssetTokenInfo(asset.id) } returns assetInfoFlow
        every { getAssetLinks(asset.id) } returns linksFlow
        every { getAssetMarket(asset.id) } returns marketFlow
        every { getPriceAlerts(asset.id) } returns MutableStateFlow<List<PriceAlertDataAggregate>>(emptyList())
        every { chartService.sections(any(), any(), any(), any(), any()) } returns emptyList()
    }

    @After
    fun tearDown() {
        viewModels.forEach { it.viewModelScope.cancel() }
        viewModels.clear()
        Dispatchers.resetMain()
    }

    @Test
    fun `a stored asset gives the scene its sections and title before any flow emits`() = runTest(testDispatcher) {
        walletAssetsFlow.value = listOf(mockAssetInfo(asset))
        every { chartService.sections(asset.toGem(), any(), null, any(), any()) } returns listOf(
            GemChartSection.Market(rows = listOf(GemAssetMarketRow.Contract(tokenId = requireNotNull(asset.id.tokenId), explorer = null))),
        )

        val viewModel = createViewModel()

        val uiModel = requireNotNull(viewModel.marketUIModel.value)
        assertEquals(asset.chain, uiModel.chain)
        assertEquals(1, uiModel.sections.size)
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
    fun `repo updates populate the sections without changing bootstrap asset`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        advanceUntilIdle()

        val market = mockAssetMarket(marketCap = 1234.0)
        val link = mockAssetLink()
        every { chartService.sections(asset.toGem(), any(), market.toGem(), any(), listOf(link.toGem())) } returns listOf(
            GemChartSection.Market(rows = listOf(GemAssetMarketRow.MarketCap(value = 1234.0, rank = null))),
            GemChartSection.Links(links = listOf(link.toGem())),
        )
        linksFlow.value = listOf(link)
        marketFlow.value = market
        currencyFlow.value = Currency.EUR

        val uiModel = viewModel.marketUIModel.first { it?.sections?.size == 2 && it.currency == Currency.EUR }!!

        assertEquals(asset.chain, uiModel.chain)
        assertEquals(Currency.EUR, uiModel.currency)
    }

    @Test
    fun `the stored price and alerts reach core untouched`() = runTest(testDispatcher) {
        val alert = mockPriceAlert(assetId = asset.id)
        assetInfoFlow.value = mockAssetInfo(asset).copy(price = mockAssetPriceInfo(price = 2.5))
        every { getPriceAlerts(asset.id) } returns MutableStateFlow(listOf(mockk<PriceAlertDataAggregate> { every { priceAlert } returns alert }))

        createViewModel()
        advanceUntilIdle()

        verify { chartService.sections(asset.toGem(), 2.5, null, listOf(alert.toGem()), emptyList()) }
    }

    private fun createViewModel(): AssetChartViewModel = AssetChartViewModel(
        getAssetTokenInfo = getAssetTokenInfo,
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
