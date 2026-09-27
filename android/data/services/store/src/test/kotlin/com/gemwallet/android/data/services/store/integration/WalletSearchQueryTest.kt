package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAccount
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbAssetList
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbSearch
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.WalletSearchQuery
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
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
class WalletSearchQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: WalletSearchQuery

    private val usdt = AssetId(Chain.Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7")
    private val usdc = AssetId(Chain.Ethereum, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = WalletSearchQuery(database.assetsDao(), database.searchDao(), database.assetListDao())
        database.walletsDao().insert(DbWallet(id = "wallet-1", name = "wallet-1", domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE, rank = 90),
                DbAsset(id = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", chain = Chain.Ethereum, name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20, rank = 80, isBuyEnabled = true),
                DbAsset(id = "ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", chain = Chain.Ethereum, name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.ERC20, rank = 70),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE, rank = 100),
            ),
        )
        database.accountsDao().insert(listOf(DbAccount(walletId = "wallet-1", derivationPath = "m/44'/60'/0'/0/0", address = "0xabc", chain = Chain.Ethereum, extendedPublicKey = null)))
        database.balancesDao().insert(
            listOf(
                DbBalance(assetId = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", walletId = "wallet-1", isVisible = true, updatedAt = 0),
                DbBalance(assetId = "ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", walletId = "wallet-1", isVisible = true, updatedAt = 0),
            ),
        )
        database.assetListDao().upsert(
            listOf(
                DbAssetList(id = "stablecoins", name = "Stablecoins", count = 2),
                DbAssetList(id = "layer1", name = "Layer 1", count = 5),
            ),
        )
        database.searchDao().insert(
            listOf(
                DbSearch(query = "stable", assetId = "ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", priority = 0),
                DbSearch(query = "stable", assetId = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", priority = 1),
                DbSearch(query = "stable", assetId = "bitcoin", priority = 2),
                DbSearch(query = "stable", listId = "layer1", priority = 1),
                DbSearch(query = "stable", listId = "stablecoins", priority = 0),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theAssetsOfAKeyAreItsPrioritisedAssetsWithinTheWalletChainsInPriorityOrder() = runBlocking(Dispatchers.IO) {
        val assets = query.assets(WalletId("wallet-1"), "stable", emptySet(), 10).first()

        assertEquals(listOf(usdc, usdt), assets.map { it.asset.id })
    }

    @Test
    fun theAssetsOfAKeyApplyTheFiltersAndTheLimit() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(usdt), query.assets(WalletId("wallet-1"), "stable", setOf(AssetsQueryFilter.Buyable), 10).first().map { it.asset.id })
        assertEquals(listOf(usdc), query.assets(WalletId("wallet-1"), "stable", emptySet(), 1).first().map { it.asset.id })
    }

    @Test
    fun aKeyWithoutPrioritiesHasNoAssets() = runBlocking(Dispatchers.IO) {
        assertEquals(emptyList<AssetId>(), query.assets(WalletId("wallet-1"), "tether", emptySet(), 10).first().map { it.asset.id })
    }

    @Test
    fun theListsOfAKeyAreInPriorityOrder() = runBlocking(Dispatchers.IO) {
        assertEquals(
            listOf(AssetList(id = "stablecoins", name = "Stablecoins", count = 2u), AssetList(id = "layer1", name = "Layer 1", count = 5u)),
            query.lists(" stable ").first(),
        )
        assertEquals(emptyList<AssetList>(), query.lists("tether").first())
    }
}
