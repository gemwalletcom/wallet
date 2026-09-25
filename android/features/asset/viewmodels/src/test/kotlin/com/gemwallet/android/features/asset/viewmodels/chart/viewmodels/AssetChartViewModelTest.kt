package com.gemwallet.android.features.asset.viewmodels.chart.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetLinks
import com.gemwallet.android.application.assets.cases.GetAssetMarket
import com.gemwallet.android.application.assets.cases.GetAssetTokenInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentCurrency
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetMarket
import com.gemwallet.android.testkit.mockAssetPrice
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockFormattedNumber
import com.gemwallet.android.testkit.mockGemSocialLink
import com.gemwallet.android.testkit.mockPriceAlert
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PriceAlertData
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
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemChartServiceInterface
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemListSectionFooter
import uniffi.gemstone.GemListSectionTitle

@OptIn(ExperimentalCoroutinesApi::class)
class AssetChartViewModelTest {
    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Solana, tokenId = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL)
    private val viewModels = mutableListOf<ViewModel>()

    private val assetInfoFlow = MutableStateFlow<AssetInfo?>(mockAssetInfo(asset = asset))
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
    private val priceAlertsQuery = mockk<PriceAlertsQuery>(relaxed = true)
    private val getCurrentCurrency = mockk<GetCurrentCurrency>(relaxed = true) {
        every { getCurrency() } returns currencyFlow
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getAssetTokenInfo(asset.id) } returns assetInfoFlow
        every { getAssetLinks(asset.id) } returns linksFlow
        every { getAssetMarket(asset.id) } returns marketFlow
        every { priceAlertsQuery(asset.id) } returns MutableStateFlow<List<PriceAlertData>>(emptyList())
        coEvery { chartService.sections(any(), any(), any(), any(), any()) } returns emptyList()
    }

    @After
    fun tearDown() {
        viewModels.forEach { it.viewModelScope.cancel() }
        viewModels.clear()
        Dispatchers.resetMain()
    }

    @Test
    fun `a stored asset gives the scene its title before any flow emits and its sections once core builds them`() = runTest(testDispatcher) {
        walletAssetsFlow.value = listOf(mockAssetInfo(asset = asset))
        coEvery { chartService.sections(asset.toGem(), any(), null, any(), any()) } returns listOf(
            section(listOf(GemListRow.Text(GemListRowTitle.TYPE, "SPL"))),
        )

        val viewModel = createViewModel()
        assertEquals(asset.name, viewModel.title.value)

        advanceUntilIdle()
        assertEquals(1, viewModel.sections.value.size)
    }

    @Test
    fun `an asset the wallet does not hold leaves the scene empty until it loads`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        assertTrue(viewModel.sections.value.isEmpty())
        assertEquals("", viewModel.title.value)
        assertEquals(asset.name, viewModel.title.first { it.isNotBlank() })
    }

    @Test
    fun `repo updates populate the sections without changing bootstrap asset`() = runTest(testDispatcher) {
        val viewModel = createViewModel()
        advanceUntilIdle()

        val market = mockAssetMarket(marketCap = 1234.0)
        val link = mockAssetLink()
        coEvery { chartService.sections(asset.toGem(), any(), market.toGem(), any(), listOf(link.toGem())) } returns listOf(
            section(listOf(GemListRow.Amount(GemListRowTitle.MARKET_CAP, mockFormattedNumber(1234.0), null))),
            section(listOf(GemListRow.Social(listOf(mockGemSocialLink()))), GemListSectionTitle.SOCIAL_LINKS),
        )
        linksFlow.value = listOf(link)
        marketFlow.value = market
        currencyFlow.value = Currency.EUR

        val sections = viewModel.sections.first { it.size == 2 }

        assertEquals(listOf(GemListRow.Amount(GemListRowTitle.MARKET_CAP, mockFormattedNumber(1234.0), null)), sections.first().rows)
        assertEquals(GemListSectionTitle.SOCIAL_LINKS, sections.last().title)
    }

    @Test
    fun `the stored price and alerts reach core untouched`() = runTest(testDispatcher) {
        val alert = mockPriceAlert(assetId = asset.id)
        assetInfoFlow.value = mockAssetInfo(asset = asset).copy(price = mockAssetPriceInfo(currency = Currency.USD, price = mockAssetPrice(price = 2.5)))
        every { priceAlertsQuery(asset.id) } returns MutableStateFlow(listOf(mockk<PriceAlertData> { every { priceAlert } returns alert }))

        createViewModel()
        advanceUntilIdle()

        coVerify { chartService.sections(asset.toGem(), 2.5, null, listOf(alert.toGem()), emptyList()) }
    }

    private fun createViewModel(): AssetChartViewModel = AssetChartViewModel(
        getAssetTokenInfo = getAssetTokenInfo,
        getAssetLinks = getAssetLinks,
        getAssetMarket = getAssetMarket,
        getWalletAssets = getWalletAssets,
        chartService = chartService,
        priceAlertsQuery = priceAlertsQuery,
        getCurrentCurrency = getCurrentCurrency,
        ioDispatcher = testDispatcher,
        assetId = asset.id,
    ).also(viewModels::add)

    private fun section(rows: List<GemListRow>, title: GemListSectionTitle = GemListSectionTitle.NONE) = GemListSection(title, GemListSectionFooter.NONE, rows)
}
