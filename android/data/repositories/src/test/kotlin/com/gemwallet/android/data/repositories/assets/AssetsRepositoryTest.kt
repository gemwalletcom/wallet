package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.mockDbAsset
import com.gemwallet.android.data.service.store.database.entities.mockDbAssetInfo
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.domains.asset.defaultAssets
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.isStakeSupported
import com.gemwallet.android.ext.isSwapSupport
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.testkit.mockAssetProperties
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetScore
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
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
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import dagger.Lazy
import uniffi.gemstone.GemAssetsService
import com.gemwallet.android.serializer.toJson
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemPriceService
import uniffi.gemstone.GemAssetConfigService
import uniffi.gemstone.GemBalanceService
import com.gemwallet.android.data.repositories.gemstone.GemstoneAssetStore

class AssetsRepositoryTest {
    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val searchDao = mockk<SearchDao>(relaxed = true)
    private val assetListDao = mockk<AssetListDao>(relaxed = true)
    private val pricesDao = mockk<PricesDao>(relaxed = true)
    private val priceService = mockk<GemPriceService>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository>()
    private val searchTokensCase = mockk<SearchTokensCase>(relaxed = true)
    private val balanceService = mockk<GemBalanceService>(relaxed = true)
    private val scope = CoroutineScope(Job())
    private val sessionFlow = MutableStateFlow<com.gemwallet.android.model.Session?>(null)

    private val assetsService = mockk<GemAssetsService>(relaxed = true)

    private fun createSubject() = AssetsRepository(
        assetsDao = assetsDao,
        assetStore = GemstoneAssetStore(assetsDao, AssetsAvailabilityService(assetsDao)),
        sessionRepository = sessionRepository,
        searchTokensCase = searchTokensCase,
        balanceService = balanceService,
        scope = scope,
    )

    @After
    fun tearDown() {
        scope.cancel()
        unmockkAll()
    }

    @Test
    fun updateBuyAvailable_appliesAvailabilityDiffWithoutResettingAllRows() = runBlocking {
        coEvery { assetsDao.getBuyAvailableAssetIds() } returns listOf("bitcoin", "ethereum")

        val subject = AssetsAvailabilityService(assetsDao)
        subject.updateBuyAvailable(listOf("ethereum", "solana"))

        coVerify { assetsDao.setBuyAvailable(listOf("bitcoin"), false) }
        coVerify { assetsDao.setBuyAvailable(listOf("solana"), true) }
    }

    @Test
    fun updateSellAvailable_appliesAvailabilityDiffWithoutResettingAllRows() = runBlocking {
        coEvery { assetsDao.getSellAvailableAssetIds() } returns listOf("bitcoin", "ethereum")

        val subject = AssetsAvailabilityService(assetsDao)
        subject.updateSellAvailable(listOf("ethereum", "solana"))

        coVerify { assetsDao.setSellAvailable(listOf("bitcoin"), false) }
        coVerify { assetsDao.setSellAvailable(listOf("solana"), true) }
    }

    @Test
    fun swapSearch_includesEnabledHiddenAndUnlinkedAssets() = runBlocking {
        every { searchDao.hasAssetPriorities("") } returns flowOf(0)

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
                walletId = "wallet-1",
                query = "",
                byChains = listOf(Chain.Solana),
                byAssets = emptyList(),
            )
        } returns flowOf(
            listOf(
                mockDbAssetInfo(asset = enabledAsset, walletId = "wallet-1", visible = true),
                mockDbAssetInfo(asset = hiddenAsset, walletId = "wallet-1", visible = false),
                mockDbAssetInfo(
                    asset = unlinkedAsset,
                    walletId = null,
                    visible = false,
                    address = null,
                ),
                mockDbAssetInfo(
                    asset = disabledAsset,
                    walletId = "wallet-1",
                    visible = true,
                    assetRank = -1,
                ),
            )
        )

        val subject = AssetsSearchService(assetsDao, searchDao, assetListDao, sessionRepository)
        val result = subject.swapSearch(
            wallet = wallet,
            query = "",
            byChains = listOf(Chain.Solana),
            byAssets = emptyList(),
        ).first()

        assertEquals(listOf(enabledAsset.id, hiddenAsset.id, unlinkedAsset.id), result.map { it.asset.id })
    }

    @Test
    fun swapSearch_usesPriorityDaoAndPreservesOrderWhenPrioritiesExist() = runBlocking {
        every { searchDao.hasAssetPriorities("usd") } returns flowOf(2)

        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Solana)),
        )
        val highPriorityAsset = mockAssetSolana()
        val lowPriorityAsset = mockAssetSolanaUSDC()

        every {
            assetsDao.swapSearchWithPriority(
                walletId = "wallet-1",
                query = "usd",
                byChains = listOf(Chain.Solana),
                byAssets = emptyList(),
            )
        } returns flowOf(
            listOf(
                mockDbAssetInfo(asset = highPriorityAsset, walletId = "wallet-1", visible = true),
                mockDbAssetInfo(asset = lowPriorityAsset, walletId = "wallet-1", visible = true),
            )
        )

        val subject = AssetsSearchService(assetsDao, searchDao, assetListDao, sessionRepository)
        val result = subject.swapSearch(
            wallet = wallet,
            query = "usd",
            byChains = listOf(Chain.Solana),
            byAssets = emptyList(),
        ).first()

        assertEquals(listOf(highPriorityAsset.id, lowPriorityAsset.id), result.map { it.asset.id })
    }

    @Test
    fun getAssetsInfo_returnsStoreRowsWithoutRepositoryDedupe() = runBlocking {
        sessionFlow.value = mockSession(wallet = mockWallet(id = "wallet-1"))
        every { sessionRepository.session() } returns sessionFlow

        val asset = mockAssetSolana()
        every { assetsDao.getAssetsInfo("wallet-1") } returns flowOf(
            listOf(
                mockDbAssetInfo(asset = asset, address = "first-address"),
                mockDbAssetInfo(asset = asset, address = "duplicate-address"),
            )
        )

        val subject = createSubject()
        val result = subject.getAssetsInfo().first()

        assertEquals(listOf(asset.id, asset.id), result.map { it.asset.id })
    }

}
