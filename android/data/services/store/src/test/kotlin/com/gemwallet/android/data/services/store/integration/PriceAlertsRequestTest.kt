package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.DbPriceAlert
import com.gemwallet.android.data.services.store.requests.PriceAlertsRequest
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Price
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class PriceAlertsRequestTest {
    private lateinit var database: GemDatabase
    private lateinit var request: PriceAlertsRequest

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        request = PriceAlertsRequest(database.priceAlertsDao())
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE, rank = 10),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE, rank = 20),
            ),
        )
        database.pricesDao().insert(
            listOf(
                DbPrice(assetId = "bitcoin", value = 100.0, dayChanged = 2.0, currency = Currency.USD, updatedAt = 5),
                DbPrice(assetId = "ethereum", value = 0.0, currency = Currency.USD),
            ),
        )
        database.priceAlertsDao().put(
            listOf(
                DbPriceAlert(id = "eth", assetId = "ethereum", currency = Currency.USD),
                DbPriceAlert(id = "btc", assetId = "bitcoin", currency = Currency.USD),
                DbPriceAlert(id = "sol", assetId = "solana", currency = Currency.USD),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun listsStoredAssetsAlertsByRankWithTheirPositivePrice() = runBlocking(Dispatchers.IO) {
        val alerts = request().first()

        assertEquals(listOf("bitcoin", "ethereum"), alerts.map { it.asset.id.chain.string })
        assertEquals(listOf(20, 10), alerts.map { it.rankScore })
        assertEquals(listOf(Price(price = 100.0, priceChangePercentage24h = 2.0, updatedAt = 5), null), alerts.map { it.price })
    }

    @Test
    fun filtersByAsset() = runBlocking(Dispatchers.IO) {
        val alerts = request(AssetId(Chain.Ethereum)).first()

        assertEquals(listOf(AssetId(Chain.Ethereum)), alerts.map { it.priceAlert.assetId })
    }
}
