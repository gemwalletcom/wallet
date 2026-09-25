package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbAssetLink
import com.gemwallet.android.data.services.store.database.entities.DbAssetMarket
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.DbPriceAlert
import com.gemwallet.android.data.services.store.queries.PriceQuery
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.PriceAlert
import com.wallet.core.primitives.PriceData
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
class PriceQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: PriceQuery
    private val ethereum = AssetId(Chain.Ethereum)
    private val bitcoin = AssetId(Chain.Bitcoin)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = PriceQuery(database.pricesDao())
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE),
            ),
        )
        database.pricesDao().insert(
            listOf(
                DbPrice(assetId = "ethereum", value = 2000.0, dayChanged = 1.5, currency = Currency.USD, updatedAt = 10),
                DbPrice(assetId = "bitcoin", value = 0.0, dayChanged = 2.0, currency = Currency.USD, updatedAt = 10),
            ),
        )
        database.assetsDao().addLinks(listOf(DbAssetLink(assetId = "ethereum", name = "website", url = "https://ethereum.org")))
        database.assetsDao().setMarket(DbAssetMarket(assetId = "ethereum", marketCap = 1234.0))
        database.priceAlertsDao().put(
            listOf(
                DbPriceAlert(id = "eth", assetId = "ethereum", currency = Currency.USD, price = 2500.0),
                DbPriceAlert(id = "btc", assetId = "bitcoin", currency = Currency.USD),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theAssetCarriesItsPriceMarketLinksAndAlerts() = runBlocking(Dispatchers.IO) {
        assertEquals(
            PriceData(
                asset = Asset(id = ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE),
                price = Price(price = 2000.0, priceChangePercentage24h = 1.5, updatedAt = 10),
                priceAlerts = listOf(PriceAlert(assetId = ethereum, currency = Currency.USD, price = 2500.0)),
                market = AssetMarket(marketCap = 1234.0),
                links = listOf(AssetLink(name = "website", url = "https://ethereum.org")),
            ),
            query(ethereum).first(),
        )
    }

    @Test
    fun aZeroPriceIsNoPrice() = runBlocking(Dispatchers.IO) {
        val data = query(bitcoin).first()

        assertEquals(bitcoin, data?.asset?.id)
        assertNull(data?.price)
        assertNull(data?.market)
    }

    @Test
    fun anAssetMissingFromTheStoreIsNull() = runBlocking(Dispatchers.IO) {
        assertNull(query(AssetId(Chain.Solana)).first())
    }
}
