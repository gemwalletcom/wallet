package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.toDB
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockPerpetual
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualMetadata
import com.wallet.core.primitives.PerpetualProvider
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
class PerpetualQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = PerpetualQuery(database.perpetualDao())
    private val bitcoinAsset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8)
    private val ethereumAsset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val bitcoin = PerpetualData(
        perpetual = mockPerpetual(
            price = 95420.5,
            pricePercentChange24h = -1.25,
            volume24h = 123456789.5,
            funding = 0.0001,
        ).copy(id = PerpetualId(PerpetualProvider.Hypercore, "BTC"), name = "BTC", assetId = bitcoinAsset.id, identifier = "0", openInterest = 42.5, maxLeverage = 40u, isIsolatedOnly = true),
        asset = bitcoinAsset,
        metadata = PerpetualMetadata(isPinned = true),
    )
    private val ethereum = PerpetualData(
        perpetual = mockPerpetual(price = 3200.25).copy(id = PerpetualId(PerpetualProvider.Hypercore, "ETH"), name = "ETH", assetId = ethereumAsset.id, identifier = "1"),
        asset = ethereumAsset,
        metadata = PerpetualMetadata(isPinned = false),
    )

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        listOf(bitcoin, ethereum).forEach { database.assetsDao().insert(it.asset.toRecord()) }
        database.perpetualDao().insert(listOf(bitcoin, ethereum).map { it.perpetual.toDB(isPinned = it.metadata.isPinned) })
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theMarketIsFoundByItsAssetWithItsExactValues() = runBlocking(Dispatchers.IO) {
        assertEquals(bitcoin, query(bitcoinAsset.id).first())
        assertEquals(ethereum, query(ethereumAsset.id).first())
    }

    @Test
    fun theMarketIsFoundByItsIdWithItsExactValues() = runBlocking(Dispatchers.IO) {
        assertEquals(bitcoin, query(bitcoin.perpetual.id).first())
        assertEquals(ethereum, query(ethereum.perpetual.id).first())
    }

    @Test
    fun anUnknownAssetOrIdHasNoMarket() = runBlocking(Dispatchers.IO) {
        assertNull(query(mockAssetId(chain = Chain.Solana)).first())
        assertNull(query(PerpetualId(PerpetualProvider.Hypercore, "SOL")).first())
    }
}
