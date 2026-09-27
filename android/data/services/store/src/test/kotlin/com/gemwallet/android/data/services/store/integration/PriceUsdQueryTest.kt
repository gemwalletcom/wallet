package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.queries.PriceUsdQuery
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class PriceUsdQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: PriceUsdQuery

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = PriceUsdQuery(database.pricesDao())
        database.pricesDao().insert(
            listOf(
                DbPrice(assetId = "ethereum", value = 1800.0, usdValue = 2000.0, currency = Currency.EUR),
                DbPrice(assetId = "bitcoin", value = 90000.0, usdValue = 100000.0, currency = Currency.EUR),
                DbPrice(assetId = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", value = 0.9, usdValue = null, currency = Currency.EUR),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theAssetPriceIsInUsdWhateverTheStoredCurrency() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(2000.0, 100000.0), listOf(query(AssetId(Chain.Ethereum)).first(), query(AssetId(Chain.Bitcoin)).first()))
    }

    @Test
    fun aPriceWithoutAUsdValueIsNull() = runBlocking(Dispatchers.IO) {
        assertNull(query(AssetId(Chain.Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7")).first())
    }

    @Test
    fun anAssetWithoutAPriceIsNull() = runBlocking(Dispatchers.IO) {
        assertNull(query(AssetId(Chain.Solana)).first())
    }
}
