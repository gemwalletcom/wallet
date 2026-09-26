package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbRecentActivity
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.RecentActivityType
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class RecentActivityQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: RecentActivityQuery

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = RecentActivityQuery(database.assetsDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE, isSwapEnabled = true),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE),
                DbAsset(id = "solana", chain = Chain.Solana, name = "Solana", symbol = "SOL", decimals = 9, type = AssetType.NATIVE, isSwapEnabled = true),
                DbAsset(id = "ethereum_0x0000000000000000000000000000000000000001", chain = Chain.Ethereum, name = "Spam", symbol = "SPAM", decimals = 18, type = AssetType.ERC20, rank = -1),
            ),
        )
        database.balancesDao().insert(
            listOf(
                DbBalance(assetId = "bitcoin", walletId = "wallet-1", available = "10000000", availableAmount = 0.1, totalAmount = 0.1, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "solana", walletId = "wallet-2", available = "1000000000", availableAmount = 1.0, totalAmount = 1.0, isVisible = true, updatedAt = 0),
            ),
        )
        listOf(
            DbRecentActivity(assetId = "ethereum", walletId = "wallet-1", type = RecentActivityType.Transfer, addedAt = 100),
            DbRecentActivity(assetId = "ethereum", walletId = "wallet-1", type = RecentActivityType.Swap, addedAt = 400),
            DbRecentActivity(assetId = "bitcoin", walletId = "wallet-1", type = RecentActivityType.Receive, addedAt = 300),
            DbRecentActivity(assetId = "solana", walletId = "wallet-1", type = RecentActivityType.Search, addedAt = 200),
            DbRecentActivity(assetId = "ethereum_0x0000000000000000000000000000000000000001", walletId = "wallet-1", type = RecentActivityType.Transfer, addedAt = 500),
            DbRecentActivity(assetId = "bitcoin", walletId = "wallet-2", type = RecentActivityType.Transfer, addedAt = 900),
        ).forEach { database.assetsDao().addRecentActivity(it) }
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun rankedAssetsOfTheWalletAreNewestFirstByTheirLatestActivity() = runBlocking(Dispatchers.IO) {
        val recents = query(WalletId("wallet-1")).first()

        assertEquals(listOf(AssetId(Chain.Ethereum), AssetId(Chain.Bitcoin), AssetId(Chain.Solana)), recents.map { it.asset.id })
        assertEquals(listOf(400L, 300L, 200L), recents.map { it.createdAt })
    }

    @Test
    fun theTypesNarrowTheActivityThatCounts() = runBlocking(Dispatchers.IO) {
        val recents = query(WalletId("wallet-1"), types = listOf(RecentActivityType.Transfer, RecentActivityType.Search)).first()

        assertEquals(listOf(AssetId(Chain.Solana), AssetId(Chain.Ethereum)), recents.map { it.asset.id })
        assertEquals(listOf(200L, 100L), recents.map { it.createdAt })
    }

    @Test
    fun theFiltersApplyToTheAssetAndTheWalletBalance() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(AssetId(Chain.Ethereum), AssetId(Chain.Solana)), query(WalletId("wallet-1"), filters = setOf(AssetsQueryFilter.Swappable)).first().map { it.asset.id })
        assertEquals(listOf(AssetId(Chain.Bitcoin)), query(WalletId("wallet-1"), filters = setOf(AssetsQueryFilter.HasBalance)).first().map { it.asset.id })
        assertEquals(listOf(AssetId(Chain.Solana)), query(WalletId("wallet-1"), filters = setOf(AssetsQueryFilter.Chains(listOf(Chain.Solana)))).first().map { it.asset.id })
    }

    @Test
    fun theLimitCapsTheRowsAndZeroReturnsEveryRow() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(AssetId(Chain.Ethereum)), query(WalletId("wallet-1"), limit = 1).first().map { it.asset.id })
        assertEquals(3, query(WalletId("wallet-1"), limit = 0).first().size)
    }

    @Test
    fun anotherWalletSeesOnlyItsOwnActivity() = runBlocking(Dispatchers.IO) {
        val recents = query(WalletId("wallet-2")).first()

        assertEquals(listOf(AssetId(Chain.Bitcoin)), recents.map { it.asset.id })
        assertEquals(listOf(900L), recents.map { it.createdAt })
    }
}
