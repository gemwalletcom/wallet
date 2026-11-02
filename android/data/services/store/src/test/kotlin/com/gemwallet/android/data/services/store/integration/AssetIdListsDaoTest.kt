package com.gemwallet.android.data.services.store.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.mockGemDatabase
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

private const val NEWEST_SQLITE_VARIABLE_LIMIT = 32_766

@RunWith(AndroidJUnit4::class)
class AssetIdListsDaoTest {
    private lateinit var database: GemDatabase
    private val wallet = "wallet-1"
    private val unlisted = "ethereum_unlisted"
    private val listed = (0..NEWEST_SQLITE_VARIABLE_LIMIT).map { "ethereum_$it" }
    private val stored = setOf(listed.first(), listed.last())

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = mockGemDatabase()
        database.walletsDao().insert(DbWallet(id = wallet, name = wallet, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        database.assetsDao().insert(
            (stored + unlisted).map { id ->
                DbAsset(id = id, chain = Chain.Ethereum, name = id, symbol = id, decimals = 18, type = AssetType.ERC20, isSwapEnabled = id == unlisted, isBuyEnabled = id == unlisted, isSellEnabled = id == unlisted)
            },
        )
        database.balancesDao().insert(stored.map { DbBalance(assetId = it, walletId = wallet, isVisible = true, updatedAt = 0) })
        database.pricesDao().insert(stored.map { DbPrice(assetId = it, currency = Currency.USD) })
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun onlyTheListedAssetsStayEnabled() = runBlocking(Dispatchers.IO) {
        val assets = database.assetsDao()

        assets.setSwappableAssets(listed)
        assets.setBuyableAssets(listed)
        assets.setSellableAssets(listed)

        val rows = assets.getAssetsByIds(listed + unlisted)
        assertEquals(stored + unlisted, rows.map { it.id }.toSet())
        assertEquals(stored, rows.filter { it.isSwapEnabled }.map { it.id }.toSet())
        assertEquals(stored, rows.filter { it.isBuyEnabled }.map { it.id }.toSet())
        assertEquals(stored, rows.filter { it.isSellEnabled }.map { it.id }.toSet())
    }

    @Test
    fun aListPastTheVariableLimitFindsRowsInEveryChunk() = runBlocking(Dispatchers.IO) {
        assertEquals(stored, database.assetsDao().getAssetIds(listed).toSet())
        assertEquals(stored, database.balancesDao().getAssetIds(wallet, listed).toSet())
        assertEquals(stored, database.balancesDao().getByAssets(wallet, listed).map { it.assetId }.toSet())
        assertEquals(stored, database.pricesDao().getByAssets(listed).map { it.assetId }.toSet())
    }

    @Test
    fun aListPastTheVariableLimitConfiguresBalancesInEveryChunk() = runBlocking(Dispatchers.IO) {
        database.assetsDao().setAssetConfiguration(wallet, listed, isVisible = false, isPinned = null)

        assertEquals(emptyList<String>(), database.balancesDao().getEnabledAssetIds(wallet))
    }
}
