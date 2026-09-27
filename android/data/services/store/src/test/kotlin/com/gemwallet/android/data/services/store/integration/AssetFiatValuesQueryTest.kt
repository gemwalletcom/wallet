package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.AssetFiatValuesQuery
import com.wallet.core.primitives.AssetFiatValue
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

@RunWith(AndroidJUnit4::class)
class AssetFiatValuesQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: AssetFiatValuesQuery

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = AssetFiatValuesQuery(database.assetsDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE),
                DbAsset(id = "tron", chain = Chain.Tron, name = "Tron", symbol = "TRX", decimals = 6, type = AssetType.NATIVE),
                DbAsset(id = "solana", chain = Chain.Solana, name = "Solana", symbol = "SOL", decimals = 9, type = AssetType.NATIVE),
                DbAsset(id = "ethereum_0x0000000000000000000000000000000000000001", chain = Chain.Ethereum, name = "Spam", symbol = "SPAM", decimals = 18, type = AssetType.ERC20, rank = -1),
            ),
        )
        database.balancesDao().insert(
            listOf(
                DbBalance(assetId = "ethereum", walletId = "wallet-1", totalAmount = 2.0, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "bitcoin", walletId = "wallet-1", totalAmount = 0.1, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "tron", walletId = "wallet-1", totalAmount = 30.0, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "solana", walletId = "wallet-1", totalAmount = 4.0, isVisible = false, updatedAt = 0),
                DbBalance(assetId = "ethereum_0x0000000000000000000000000000000000000001", walletId = "wallet-1", totalAmount = 100.0, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "ethereum", walletId = "wallet-2", totalAmount = 7.0, isVisible = true, updatedAt = 0),
            ),
        )
        database.pricesDao().insert(
            listOf(
                DbPrice(assetId = "ethereum", value = 2000.0, dayChanged = 1.5, currency = Currency.USD),
                DbPrice(assetId = "bitcoin", value = 60000.0, dayChanged = -2.0, currency = Currency.USD),
                DbPrice(assetId = "solana", value = 150.0, dayChanged = 3.0, currency = Currency.USD),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theWalletsVisibleRankedAssetsGiveTheirAmountPriceAndChange() = runBlocking(Dispatchers.IO) {
        assertEquals(
            setOf(
                AssetFiatValue(amount = 2.0, price = 2000.0, priceChangePercentage24h = 1.5),
                AssetFiatValue(amount = 0.1, price = 60000.0, priceChangePercentage24h = -2.0),
                AssetFiatValue(amount = 30.0, price = 0.0, priceChangePercentage24h = 0.0),
            ),
            query(WalletId("wallet-1")).first().toSet(),
        )
    }

    @Test
    fun anotherWalletGivesOnlyItsOwnBalances() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(AssetFiatValue(amount = 7.0, price = 2000.0, priceChangePercentage24h = 1.5)), query(WalletId("wallet-2")).first())
    }
}
