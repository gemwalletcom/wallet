package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.RoomStoreTransactionRunner
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

/**
 * The paired half of BalanceStoreTests on iOS: the same six contract cases, against the real DAO
 * and the real transaction runner.
 */
@RunWith(AndroidJUnit4::class)
class BalancesDaoTest {
    private lateinit var database: GemDatabase
    private val wallet = "wallet-1"
    private val other = "wallet-2"

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        for (id in listOf(wallet, other)) {
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        for (chain in listOf(Chain.Ethereum, Chain.Solana)) {
            database.assetsDao().insert(DbAsset(id = chain.string, chain = chain, name = chain.string, symbol = chain.string, decimals = 18, type = AssetType.NATIVE))
        }
    }

    @After
    fun tearDown() = database.close()

    private fun balance(assetId: String, walletId: String = wallet, visible: Boolean = true) = DbBalance(assetId = assetId, walletId = walletId, isVisible = visible, updatedAt = 0)

    @Test
    fun addingABalanceTwiceKeepsTheStoredOne() = runBlocking(Dispatchers.IO) {
        database.balancesDao().insertIgnore(balance("ethereum"))
        database.balancesDao().insert(
            balance("ethereum").apply {
                available = "5"
                availableAmount = 5.0
            },
        )

        database.balancesDao().insertIgnore(balance("ethereum", visible = false))

        val stored = database.balancesDao().getByAssets(wallet, listOf("ethereum")).single()
        assertEquals("5", stored.available)
        assertTrue("a second add never flips the configuration", stored.isVisible)
    }

    @Test
    fun updatingABalanceThatHasNoRowWritesNothing() = runBlocking(Dispatchers.IO) {
        database.balancesDao().updateBalance(
            walletId = wallet, assetId = "ethereum",
            available = "5", availableAmount = 5.0,
            frozen = "0", frozenAmount = 0.0, locked = "0", lockedAmount = 0.0, staked = "0", stakedAmount = 0.0,
            pending = "0", pendingAmount = 0.0, pendingUnconfirmed = "0", pendingUnconfirmedAmount = 0.0,
            rewards = "0", rewardsAmount = 0.0, reserved = "0", reservedAmount = 0.0,
            withdrawable = "0", withdrawableAmount = 0.0, earn = "0", earnAmount = 0.0,
            metadata = null, isActive = true, updatedAt = 1,
        )

        assertTrue(database.balancesDao().getByAssets(wallet, listOf("ethereum")).isEmpty())
    }

    @Test
    fun aWalletNeverSeesAnotherWalletsBalance() = runBlocking(Dispatchers.IO) {
        database.balancesDao().insert(balance("ethereum"))
        database.balancesDao().insert(balance("ethereum", walletId = other))

        database.assetsDao().setAssetConfiguration(wallet, listOf("ethereum"), isVisible = false, isPinned = null)

        assertEquals(listOf("ethereum"), database.balancesDao().getEnabledAssetIds(other))
        assertEquals(emptyList<String>(), database.balancesDao().getEnabledAssetIds(wallet))
    }

    @Test
    fun aConfigurationThatChangesNothingWritesNothing() = runBlocking(Dispatchers.IO) {
        database.balancesDao().insert(balance("ethereum"))
        val before = database.balancesDao().getByAssets(wallet, listOf("ethereum")).single().updatedAt

        database.assetsDao().setAssetConfiguration(wallet, listOf("ethereum"), isVisible = true, isPinned = false)

        val unchanged = database.balancesDao().getByAssets(wallet, listOf("ethereum")).single()
        assertEquals(before, unchanged.updatedAt)
        assertTrue(unchanged.isVisible)

        database.assetsDao().setAssetConfiguration(wallet, listOf("ethereum"), isVisible = false, isPinned = null)
        assertEquals(false, database.balancesDao().getByAssets(wallet, listOf("ethereum")).single().isVisible)
    }

    @Test
    fun onlyTheRowsAskedForComeBack() = runBlocking(Dispatchers.IO) {
        database.balancesDao().insert(listOf(balance("ethereum"), balance("solana")))

        assertEquals(listOf("ethereum"), database.balancesDao().getByAssets(wallet, listOf("ethereum")).map { it.assetId })
        assertEquals(2, database.balancesDao().getByAssets(wallet, listOf("ethereum", "solana")).size)
    }

    @Test
    fun aBatchThatFailsPartWayLeavesNothingBehind() = runBlocking(Dispatchers.IO) {
        val runner = RoomStoreTransactionRunner(database)

        runCatching {
            runner.run {
                database.balancesDao().insert(balance("ethereum"))
                error("the publication lane gave up half way")
            }
        }

        assertTrue("the row written before the failure is rolled back with it", database.balancesDao().getByAssets(wallet, listOf("ethereum")).isEmpty())
    }

    @Test
    fun aBatchReachesAnObserverWholeOrNotAtAll() = runBlocking(Dispatchers.IO) {
        val runner = RoomStoreTransactionRunner(database)
        runner.run {
            database.balancesDao().insert(listOf(balance("ethereum"), balance("solana")))
        }

        val observed = database.assetsDao().getAssetsInfo(wallet).first()

        assertEquals("both rows of the batch are there, never one", 2, observed.count { it.balanceAvailable != null })
        assertNull("a balance with nothing to carry has no metadata", database.balancesDao().getByAssets(wallet, listOf("ethereum")).single().metadata)
    }
}
