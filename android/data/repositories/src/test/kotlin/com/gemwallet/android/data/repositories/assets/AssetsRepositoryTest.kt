package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.DbAssetConfig
import com.gemwallet.android.data.service.store.database.entities.DbFiatRate
import com.gemwallet.android.data.service.store.database.entities.DbAssetLink
import com.gemwallet.android.data.service.store.database.entities.DbAssetMarket
import com.gemwallet.android.data.service.store.database.entities.DbAssetWallet
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.service.store.database.entities.mockDbAsset
import com.gemwallet.android.data.service.store.database.entities.mockDbAssetWallet
import com.gemwallet.android.data.service.store.database.entities.mockDbAssetInfo
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.isStakeSupported
import com.gemwallet.android.ext.isSwapSupport
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetFull
import com.gemwallet.android.testkit.mockAssetLink
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetMarket
import com.gemwallet.android.testkit.mockAssetProperties
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockChartValuePercentage
import com.gemwallet.android.testkit.mockPrice
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.slot
import io.mockk.unmockkAll
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.assetDefaultRank

class AssetsRepositoryTest {

    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val assetsPriorityDao = mockk<AssetsPriorityDao>(relaxed = true)
    private val balancesDao = mockk<BalancesDao>(relaxed = true)
    private val pricesDao = mockk<PricesDao>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>()
    private val balancesService = mockk<BalancesService>(relaxed = true)
    private val getChangedTransactions = mockk<GetChangedTransactions>()
    private val searchTokensCase = mockk<SearchTokensCase>(relaxed = true)
    private val streamSubscriptionService = mockk<StreamSubscriptionService>(relaxed = true)
    private val updateBalances = mockk<UpdateBalances>(relaxed = true)
    private val scope = CoroutineScope(Job())
    private val sessionFlow = MutableStateFlow<com.gemwallet.android.model.Session?>(null)

    private fun createSubject() = AssetsRepository(
        assetsDao = assetsDao,
        assetsPriorityDao = assetsPriorityDao,
        balancesDao = balancesDao,
        pricesDao = pricesDao,
        sessionRepository = sessionRepository,
        balancesService = balancesService,
        getChangedTransactions = getChangedTransactions,
        searchTokensCase = searchTokensCase,
        streamSubscriptionService = streamSubscriptionService,
        updateBalances = updateBalances,
        scope = scope,
    )

    @After
    fun tearDown() {
        scope.cancel()
        unmockkAll()
    }

    @Test
    fun updateAssetMetadata_storesLinksPriceAndMarketFromAssetResponse() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow

        val asset = mockAssetSolana()
        val assetFull = mockAssetFull(
            asset = asset,
            properties = mockAssetProperties(
                isSwapable = false,
            ),
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

        coEvery { sessionRepository.getCurrentCurrency() } returns Currency.EUR
        every { pricesDao.getRates(Currency.EUR) } returns flowOf(DbFiatRate(Currency.EUR, 0.5))
        coEvery { pricesDao.getByAssets(listOf("solana")) } returns emptyList()

        val subject = createSubject()
        subject.updateAssetMetadata(assetFull)

        val linksSlot = slot<List<DbAssetLink>>()
        val assetSlot = slot<DbAsset>()
        val priceSlot = slot<DbPrice>()
        val marketSlot = slot<DbAssetMarket>()

        coVerify { assetsDao.update(capture(assetSlot)) }
        coVerify { assetsDao.addLinks(capture(linksSlot)) }
        coVerify { pricesDao.insert(capture(priceSlot)) }
        coVerify { assetsDao.setMarket(capture(marketSlot)) }

        assertEquals("solana", assetSlot.captured.id)
        assertEquals(false, assetSlot.captured.isSwapEnabled)
        assertEquals(1, linksSlot.captured.size)
        assertEquals("website", linksSlot.captured.single().name)
        assertEquals("https://bitcoin.org", linksSlot.captured.single().url)

        assertEquals("solana", priceSlot.captured.assetId)
        assertEquals(50.0, priceSlot.captured.value ?: 0.0, 0.0)
        assertEquals(100.0, priceSlot.captured.usdValue ?: 0.0, 0.0)
        assertEquals("EUR", priceSlot.captured.currency)

        assertEquals("solana", marketSlot.captured.assetId)
        assertEquals(500.0, marketSlot.captured.marketCap ?: 0.0, 0.0)
        assertEquals(750.0, marketSlot.captured.marketCapFdv ?: 0.0, 0.0)
        assertEquals(100.0, marketSlot.captured.totalVolume ?: 0.0, 0.0)
        assertEquals(150.0, marketSlot.captured.allTimeHigh ?: 0.0, 0.0)
        assertEquals(25.0, marketSlot.captured.allTimeLow ?: 0.0, 0.0)
    }

    @Test
    fun updateBuyAvailable_appliesAvailabilityDiffWithoutResettingAllRows() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow
        coEvery { assetsDao.getSwapAvailableAssetIds(any()) } returns emptyList()
        coEvery { assetsDao.getBuyAvailableAssetIds() } returns listOf("bitcoin", "ethereum")

        val subject = createSubject()
        subject.updateBuyAvailable(listOf("ethereum", "solana"))

        coVerify { assetsDao.setBuyAvailable(listOf("bitcoin"), false) }
        coVerify { assetsDao.setBuyAvailable(listOf("solana"), true) }
    }

    @Test
    fun updateSellAvailable_appliesAvailabilityDiffWithoutResettingAllRows() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow
        coEvery { assetsDao.getSwapAvailableAssetIds(any()) } returns emptyList()
        coEvery { assetsDao.getSellAvailableAssetIds() } returns listOf("bitcoin", "ethereum")

        val subject = createSubject()
        subject.updateSellAvailable(listOf("ethereum", "solana"))

        coVerify { assetsDao.setSellAvailable(listOf("bitcoin"), false) }
        coVerify { assetsDao.setSellAvailable(listOf("solana"), true) }
    }

    @Test
    fun addApiAsset_insertsApiRank() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow

        val asset = mockAssetSolana()
        val assetBasic = AssetBasic(
            asset = asset,
            properties = mockAssetProperties(
                isSwapable = false,
                isStakeable = true,
                stakingApr = 4.2,
            ),
            score = AssetScore(rank = 321),
        )

        val subject = createSubject()
        subject.add(
            walletId = "wallet-1",
            accountAddress = "wallet-address",
            asset = assetBasic,
            visible = true,
        )

        val assetSlot = slot<DbAsset>()
        val updateSlot = slot<DbAssetBasicUpdate>()
        val linkSlot = slot<DbAssetWallet>()
        val configSlot = slot<DbAssetConfig>()

        coVerify { assetsDao.insert(capture(assetSlot), capture(linkSlot), capture(configSlot)) }
        coVerify { assetsDao.updateBasicAsset(capture(updateSlot)) }

        assertEquals(321, assetSlot.captured.rank)
        assertEquals(false, assetSlot.captured.isSwapEnabled)
        assertEquals(mockDbAssetWallet(assetId = asset.id, accountAddress = "wallet-address"), linkSlot.captured)
        assertEquals(true, configSlot.captured.isVisible)
        assertEquals(321, updateSlot.captured.rank)
        assertEquals(false, updateSlot.captured.isSwapEnabled)
        assertEquals(true, updateSlot.captured.isStakeEnabled)
        assertEquals(4.2, updateSlot.captured.stakingApr ?: 0.0, 0.0)
    }

    @Test
    fun addLocalAsset_insertsDefaultTokenRank() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow

        val asset = mockAssetSolanaUSDC()

        val subject = createSubject()
        subject.add(
            walletId = "wallet-1",
            accountAddress = "wallet-address",
            asset = asset,
            visible = true,
        )

        val assetSlot = slot<DbAsset>()
        coVerify { assetsDao.insert(capture(assetSlot), any(), any()) }
        coVerify(exactly = 0) { assetsDao.updateAssetRank(any(), any()) }
        coVerify(exactly = 0) { assetsDao.updateBasicAsset(any()) }

        assertEquals(15, assetSlot.captured.rank)
        assertEquals(asset.defaultBasic.score.rank, assetSlot.captured.rank)
    }

    @Test
    fun updateNativeAssetRanks_repairsLegacyNativeRanks() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        mockkStatic("uniffi.gemstone.GemstoneKt")
        every { Chain.available() } returns setOf(Chain.Solana, Chain.Ethereum)
        every { Chain.Solana.asset() } returns mockAssetSolana()
        every { Chain.Ethereum.asset() } returns mockAssetEthereum()
        every { Chain.Solana.isSwapSupport() } returns true
        every { Chain.Solana.isStakeSupported() } returns true
        every { Chain.Ethereum.isSwapSupport() } returns true
        every { Chain.Ethereum.isStakeSupported() } returns false
        every { assetDefaultRank(Chain.Solana.string) } returns 99
        every { assetDefaultRank(Chain.Ethereum.string) } returns 77

        val subject = createSubject()
        subject.updateNativeAssetRanks()

        coVerify { assetsDao.updateAssetRank("solana", 99) }
        coVerify { assetsDao.updateAssetRank("ethereum", 77) }
    }

    @Test
    fun swapSearch_includesEnabledHiddenAndUnlinkedAssets() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow
        every { assetsPriorityDao.hasPriorities("") } returns flowOf(0)

        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Solana)),
        )
        val enabledAsset = mockAssetSolana()
        val hiddenAsset = mockAssetSolanaUSDC()
        val unlinkedAsset = mockAssetSolanaUSDC().copy(
            id = AssetId(Chain.Solana, "jto"),
            name = "Jito",
            symbol = "JTO",
            decimals = 9,
        )
        val disabledAsset = mockAssetSolanaUSDC().copy(
            id = AssetId(Chain.Solana, "bonk"),
            name = "Bonk",
            symbol = "BONK",
            decimals = 5,
        )

        every {
            assetsDao.swapSearch(
                query = "",
                byChains = listOf(Chain.Solana),
                byAssets = emptyList(),
            )
        } returns flowOf(
            listOf(
                mockDbAssetInfo(asset = enabledAsset, walletId = "wallet-1", visible = true, sessionId = 1),
                mockDbAssetInfo(asset = hiddenAsset, walletId = "wallet-1", visible = false, sessionId = 1),
                mockDbAssetInfo(
                    asset = unlinkedAsset,
                    walletId = null,
                    visible = false,
                    sessionId = null,
                    walletName = null,
                    walletType = null,
                    address = null,
                ),
                mockDbAssetInfo(
                    asset = disabledAsset,
                    walletId = "wallet-1",
                    visible = true,
                    sessionId = 1,
                    assetRank = 0,
                ),
            )
        )

        val subject = createSubject()
        val result = subject.swapSearch(
            wallet = wallet,
            query = "",
            byChains = listOf(Chain.Solana),
            byAssets = emptyList(),
            tags = emptyList(),
        ).first()

        assertEquals(listOf(enabledAsset.id, hiddenAsset.id, unlinkedAsset.id), result.map { it.asset.id })
    }

    @Test
    fun getAssetsInfo_dedupesDuplicateAssetIdsFromStore() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow

        val asset = mockAssetSolana()
        every { assetsDao.getAssetsInfo() } returns flowOf(
            listOf(
                mockDbAssetInfo(asset = asset, address = "first-address"),
                mockDbAssetInfo(asset = asset, address = "duplicate-address"),
            )
        )

        val subject = createSubject()
        val result = subject.getAssetsInfo().first()

        assertEquals(listOf(asset.id), result.map { it.asset.id })
    }

    @Test
    fun getNativeAssets_returnsNativeWalletAssetsFromDao() = runBlocking {
        every { getChangedTransactions.getChangedTransactions() } returns emptyFlow()
        every { sessionRepository.session() } returns sessionFlow

        val wallet = mockWallet(id = "wallet-1")
        val nativeAsset = mockAssetSolana()
        every { assetsDao.getNativeWalletAssets(wallet.id) } returns flowOf(
            listOf(
                mockDbAsset(asset = nativeAsset),
            )
        )

        val subject = createSubject()
        val result = subject.getNativeAssets(wallet)

        assertEquals(listOf(nativeAsset.id), result.map { it.id })
    }
}
