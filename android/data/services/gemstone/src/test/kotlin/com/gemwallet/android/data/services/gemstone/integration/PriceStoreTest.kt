package com.gemwallet.android.data.services.gemstone.integration

import android.database.sqlite.SQLiteException
import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.entities.DbFiatRate
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.data.services.gemstone.stores.GemstonePriceStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemPriceUpdate

@RunWith(AndroidJUnit4::class)
class PriceStoreTest {
    private lateinit var database: GemDatabase
    private lateinit var store: GemstonePriceStore
    private val initialRate = DbFiatRate(Currency.EUR, 0.8)
    private val price = DbPrice(
        assetId = mockAsset().id.toIdentifier(),
        value = 80.0,
        usdValue = 100.0,
        dayChanged = 2.0,
        currency = Currency.EUR,
        updatedAt = 42,
    )

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        store = GemstonePriceStore(database.pricesDao(), database.assetsDao())
        database.pricesDao().setRates(listOf(initialRate))
        database.pricesDao().insert(price)
        database.openHelper.writableDatabase.execSQL(
            "CREATE TRIGGER reject_price_update BEFORE UPDATE ON prices BEGIN SELECT RAISE(ABORT, 'price update rejected'); END",
        )
    }

    @After
    fun tearDown() {
        database.close()
    }

    @Test
    fun failedConversionRollsBackTheTickAndIdenticalRetryCommitsIt() = runBlocking(Dispatchers.IO) {
        val conversion = FiatRate(Currency.EUR, 0.9).toGem()
        val rates = listOf(conversion, FiatRate(Currency.GBP, 0.7).toGem())
        val ethereum = mockAssetEthereum().id.toIdentifier()
        val prices = listOf(GemPriceUpdate(assetId = ethereum, price = 180.0, priceUsd = 200.0, priceChangePercentage24h = 1.0, updatedAt = 43))

        val failure = runCatching { store.saveRatesAndPrices(Currency.EUR.toGem(), rates, conversion, prices) }.exceptionOrNull()

        assertTrue(failure is SQLiteException)
        assertEquals(listOf(initialRate), database.pricesDao().getRates())
        assertEquals(listOf(price), database.pricesDao().getByAssets(listOf(price.assetId, ethereum)))

        database.openHelper.writableDatabase.execSQL("DROP TRIGGER reject_price_update")
        store.saveRatesAndPrices(Currency.EUR.toGem(), rates, conversion, prices)

        assertEquals(setOf(DbFiatRate(Currency.EUR, 0.9), DbFiatRate(Currency.GBP, 0.7)), database.pricesDao().getRates().toSet())
        assertEquals(
            setOf(price.copy(value = 90.0), DbPrice(assetId = ethereum, value = 180.0, usdValue = 200.0, dayChanged = 1.0, currency = Currency.EUR, updatedAt = 43)),
            database.pricesDao().getByAssets(listOf(price.assetId, ethereum)).toSet(),
        )
    }

    @Test
    fun rateWithoutConversionDoesNotTouchPrices() = runBlocking(Dispatchers.IO) {
        store.saveRatesAndPrices(Currency.EUR.toGem(), listOf(FiatRate(Currency.GBP, 0.7).toGem()), null, emptyList())

        assertEquals(setOf(initialRate, DbFiatRate(Currency.GBP, 0.7)), database.pricesDao().getRates().toSet())
        assertEquals(listOf(price), database.pricesDao().getByAssets(listOf(price.assetId)))
    }
}
