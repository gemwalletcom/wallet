package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAccount
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
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
import java.math.BigInteger

@RunWith(AndroidJUnit4::class)
class AssetsQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: AssetsQuery

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = AssetsQuery(database.assetsDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE, rank = 90),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE, rank = 100),
                DbAsset(id = "solana", chain = Chain.Solana, name = "Solana", symbol = "SOL", decimals = 9, type = AssetType.NATIVE, rank = 10),
                DbAsset(id = "tron", chain = Chain.Tron, name = "Tron", symbol = "TRX", decimals = 6, type = AssetType.NATIVE, rank = 50),
                DbAsset(id = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", chain = Chain.Ethereum, name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20, rank = 80),
                DbAsset(id = "ethereum_0x0000000000000000000000000000000000000001", chain = Chain.Ethereum, name = "Spam", symbol = "SPAM", decimals = 18, type = AssetType.ERC20, rank = -1),
            ),
        )
        database.accountsDao().insert(
            listOf(
                DbAccount(walletId = "wallet-1", derivationPath = "m/44'/60'/0'/0/0", address = "0xabc", chain = Chain.Ethereum, extendedPublicKey = null),
                DbAccount(walletId = "wallet-2", derivationPath = "m/44'/60'/0'/0/0", address = "0xdef", chain = Chain.Ethereum, extendedPublicKey = null),
            ),
        )
        database.balancesDao().insert(
            listOf(
                DbBalance(assetId = "ethereum", walletId = "wallet-1", available = "2000000000000000000", availableAmount = 2.0, totalAmount = 2.0, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "bitcoin", walletId = "wallet-1", available = "10000000", availableAmount = 0.1, totalAmount = 0.1, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "solana", walletId = "wallet-1", isPinned = true, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "tron", walletId = "wallet-1", isVisible = true, updatedAt = 0),
                DbBalance(assetId = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", walletId = "wallet-1", available = "5000000", availableAmount = 5.0, totalAmount = 5.0, isVisible = false, updatedAt = 0),
                DbBalance(assetId = "ethereum_0x0000000000000000000000000000000000000001", walletId = "wallet-1", totalAmount = 100.0, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "ethereum", walletId = "wallet-2", available = "7000000000000000000", availableAmount = 7.0, totalAmount = 7.0, isVisible = true, updatedAt = 0),
            ),
        )
        database.pricesDao().insert(
            listOf(
                DbPrice(assetId = "ethereum", value = 2000.0, dayChanged = 1.5, currency = Currency.USD),
                DbPrice(assetId = "bitcoin", value = 60000.0, dayChanged = -2.0, currency = Currency.USD),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun visibleRankedAssetsArePinnedFirstThenByFiatValueThenByRank() = runBlocking(Dispatchers.IO) {
        val assets = query(WalletId("wallet-1")).first()

        assertEquals(
            listOf(AssetId(Chain.Solana), AssetId(Chain.Bitcoin), AssetId(Chain.Ethereum), AssetId(Chain.Tron)),
            assets.map { it.asset.id },
        )
        assertEquals(listOf(true, false, false, false), assets.map { it.metadata?.isPinned })
    }

    @Test
    fun eachAssetCarriesTheWalletBalanceAndPrice() = runBlocking(Dispatchers.IO) {
        val bitcoin = query(WalletId("wallet-1")).first().first { it.asset.id == AssetId(Chain.Bitcoin) }

        assertEquals(BigInteger.valueOf(10_000_000), bitcoin.balance.balance.available)
        assertEquals(listOf(60000.0, -2.0), listOf(bitcoin.price?.price?.price, bitcoin.price?.price?.priceChangePercentage24h))
        assertEquals(WalletId("wallet-1"), bitcoin.walletId)
    }

    @Test
    fun anotherWalletSeesOnlyItsOwnAssets() = runBlocking(Dispatchers.IO) {
        val assets = query(WalletId("wallet-2")).first()

        assertEquals(listOf(AssetId(Chain.Ethereum)), assets.map { it.asset.id })
        assertEquals(BigInteger("7000000000000000000"), assets.single().balance.balance.available)
        assertEquals("0xdef", assets.single().owner?.address)
    }
}
