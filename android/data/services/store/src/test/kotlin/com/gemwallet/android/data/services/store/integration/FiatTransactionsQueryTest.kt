package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbFiatTransaction
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.FiatTransactionsQuery
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FiatProviderName
import com.wallet.core.primitives.FiatQuoteType
import com.wallet.core.primitives.FiatTransactionAssetData
import com.wallet.core.primitives.FiatTransactionStatus
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
class FiatTransactionsQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: FiatTransactionsQuery

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = FiatTransactionsQuery(database.fiatTransactionsDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE),
            ),
        )
        database.fiatTransactionsDao().insert(
            listOf(
                transaction(id = "older", walletId = "wallet-1", assetId = "bitcoin", createdAt = 10),
                transaction(id = "newest", walletId = "wallet-1", assetId = "ethereum", createdAt = 30),
                transaction(id = "other-wallet", walletId = "wallet-2", assetId = "ethereum", createdAt = 40),
                transaction(id = "middle", walletId = "wallet-1", assetId = "ethereum", createdAt = 20),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun listsTheWalletTransactionsNewestFirstWithTheirAsset() = runBlocking(Dispatchers.IO) {
        val transactions = query("wallet-1").first()

        assertEquals(listOf("newest", "middle", "older"), transactions.map { it.id })
        assertEquals(
            FiatTransactionAssetData(
                id = "older",
                asset = Asset(id = AssetId(Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE),
                transactionType = FiatQuoteType.Buy,
                provider = FiatProviderName.MoonPay,
                status = FiatTransactionStatus.Complete,
                fiatAmount = 100.0,
                fiatCurrency = "USD",
                value = "1000",
                createdAt = 10,
                detailsUrl = "https://provider.test/older",
            ),
            transactions.last(),
        )
    }

    @Test
    fun walletWithoutTransactionsListsNone() = runBlocking(Dispatchers.IO) {
        assertEquals(emptyList<FiatTransactionAssetData>(), query("wallet-3").first())
    }

    private fun transaction(id: String, walletId: String, assetId: String, createdAt: Long) = DbFiatTransaction(
        id = id,
        walletId = walletId,
        assetId = assetId,
        transactionType = FiatQuoteType.Buy,
        provider = FiatProviderName.MoonPay,
        status = FiatTransactionStatus.Complete,
        fiatAmount = 100.0,
        fiatCurrency = "USD",
        value = "1000",
        createdAt = createdAt,
        detailsUrl = "https://provider.test/$id",
    )
}
