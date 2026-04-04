package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbFiatRate
import com.gemwallet.android.data.service.store.database.entities.DbAssetLink
import com.gemwallet.android.data.service.store.database.entities.DbAssetMarket
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetFull
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetMarket
import com.gemwallet.android.testkit.mockChartValuePercentage
import com.gemwallet.android.testkit.mockPrice
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test

class AssetsRepositoryTest {

    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val assetsPriorityDao = mockk<AssetsPriorityDao>(relaxed = true)
    private val balancesDao = mockk<BalancesDao>(relaxed = true)
    private val pricesDao = mockk<PricesDao>(relaxed = true)
    private val gemApi = mockk<GemApiClient>()
    private val sessionRepository = mockk<SessionRepository>()
    private val balancesService = mockk<BalancesService>(relaxed = true)
    private val getChangedTransactions = mockk<GetChangedTransactions>()
    private val searchTokensCase = mockk<SearchTokensCase>(relaxed = true)
    private val priceClient = mockk<PriceWebSocketClient>(relaxed = true)
    private val updateBalances = mockk<UpdateBalances>(relaxed = true)
    private val scope = CoroutineScope(Job())
    private val sessionFlow = MutableStateFlow<com.gemwallet.android.model.Session?>(null)

    private fun createSubject() = AssetsRepository(
        assetsDao = assetsDao,
        assetsPriorityDao = assetsPriorityDao,
        balancesDao = balancesDao,
        pricesDao = pricesDao,
        gemApi = gemApi,
        sessionRepository = sessionRepository,
        balancesService = balancesService,
        getChangedTransactions = getChangedTransactions,
        searchTokensCase = searchTokensCase,
        priceClient = priceClient,
        updateBalances = updateBalances,
        scope = scope,
    )

    @After
    fun tearDown() {
        scope.cancel()
    }

    @Test
    fun syncAssetMetadata_storesLinksPriceAndMarketFromAssetResponse() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow

        val asset = mockAsset()
        val assetFull = mockAssetFull(
            asset = asset,
            links = listOf(mockAssetLink()),
            price = mockPrice(
                price = 100.0,
                priceChangePercentage24h = -5.0,
                updatedAt = 1L,
            ),
            market = mockAssetMarket(
                marketCap = 1_000.0,
                marketCapFdv = 1_500.0,
                marketCapRank = 1,
                totalVolume = 200.0,
                circulatingSupply = 10.0,
                totalSupply = 20.0,
                maxSupply = 21.0,
                allTimeHighValue = mockChartValuePercentage(date = 2L, value = 300.0f, percentage = -10.0f),
                allTimeLowValue = mockChartValuePercentage(date = 3L, value = 50.0f, percentage = 80.0f),
            ),
        )

        coEvery { gemApi.getAsset("bitcoin") } returns assetFull
        coEvery { sessionRepository.getCurrentCurrency() } returns Currency.EUR
        every { pricesDao.getRates(Currency.EUR) } returns flowOf(DbFiatRate(Currency.EUR, 0.5))
        coEvery { pricesDao.getByAssets(listOf("bitcoin")) } returns emptyList()

        val subject = createSubject()
        subject.syncAssetMetadata(asset.id)

        val linksSlot = slot<List<DbAssetLink>>()
        val priceSlot = slot<DbPrice>()
        val marketSlot = slot<DbAssetMarket>()

        coVerify { assetsDao.addLinks(capture(linksSlot)) }
        coVerify { pricesDao.insert(capture(priceSlot)) }
        coVerify { assetsDao.setMarket(capture(marketSlot)) }

        assertEquals(1, linksSlot.captured.size)
        assertEquals("website", linksSlot.captured.single().name)
        assertEquals("https://bitcoin.org", linksSlot.captured.single().url)

        assertEquals("bitcoin", priceSlot.captured.assetId)
        assertEquals(50.0, priceSlot.captured.value ?: 0.0, 0.0)
        assertEquals(100.0, priceSlot.captured.usdValue ?: 0.0, 0.0)
        assertEquals("EUR", priceSlot.captured.currency)

        assertEquals("bitcoin", marketSlot.captured.assetId)
        assertEquals(500.0, marketSlot.captured.marketCap ?: 0.0, 0.0)
        assertEquals(750.0, marketSlot.captured.marketCapFdv ?: 0.0, 0.0)
        assertEquals(100.0, marketSlot.captured.totalVolume ?: 0.0, 0.0)
        assertEquals(150.0, marketSlot.captured.allTimeHigh ?: 0.0, 0.0)
        assertEquals(25.0, marketSlot.captured.allTimeLow ?: 0.0, 0.0)
    }
}
