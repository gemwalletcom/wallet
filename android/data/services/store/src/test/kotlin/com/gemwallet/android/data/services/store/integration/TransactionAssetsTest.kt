package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TransactionAssetsTest {
    private lateinit var database: GemDatabase
    private val wallet = mockWallet()
    private val bitcoin = mockAsset()
    private val usdt = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xdac17f958d2ee523a2206206994597c13d831ec7"), name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20)
    private val swap = mockTransaction(type = TransactionType.Swap)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        database.walletsDao().insert(wallet.toRecord())
        database.assetsDao().insert(listOf(bitcoin, mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18), usdt).map { it.toRecord() })
        database.pricesDao().insert(DbPrice(assetId = usdt.id.toIdentifier(), value = 1.0, dayChanged = 0.5, currency = Currency.USD))
        database.transactionsDao().insert(listOf(swap.toRecord(wallet.id)))
        database.transactionsDao().replaceTransactionAssets(mapOf(swap.id.identifier to listOf(bitcoin.id.toIdentifier(), usdt.id.toIdentifier())))
    }

    @After
    fun tearDown() {
        database.close()
    }

    @Test
    fun aListedTransactionCarriesEveryAssetItTouches() = runBlocking(Dispatchers.IO) {
        val transaction = database.transactionsDao().getTransactionListItems(wallet.id, emptyList(), 100).first().single().toDTO()

        assertEquals(setOf(bitcoin, usdt), transaction?.assets?.toSet())
    }

    @Test
    fun theTransactionDetailCarriesThePricesOfEveryAssetItTouches() = runBlocking(Dispatchers.IO) {
        val transaction = database.transactionsDao().getExtendedTransaction(wallet.id, swap.id).first()?.toDTO()

        assertEquals(setOf(bitcoin, usdt), transaction?.assets?.toSet())
        assertEquals(listOf(AssetPrice(usdt.id, 1.0, 0.5, 0L)), transaction?.prices)
    }

    @Test
    fun theAssetFilterMatchesAnyAssetTheTransactionTouches() = runBlocking(Dispatchers.IO) {
        val transactions = database.transactionsDao()

        assertEquals(1, transactions.getTransactionListItems(wallet.id, listOf(TransactionsRequestFilter.Asset(usdt.id)), 100).first().size)
        assertEquals(0, transactions.getTransactionListItems(wallet.id, listOf(TransactionsRequestFilter.Asset(mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18).id)), 100).first().size)
    }
}
