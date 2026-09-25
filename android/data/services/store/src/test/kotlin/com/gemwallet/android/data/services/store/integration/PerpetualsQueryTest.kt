package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbSearch
import com.gemwallet.android.data.services.store.database.entities.toDB
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.PerpetualsQuery
import com.gemwallet.android.ext.toIdentifier
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
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class PerpetualsQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = PerpetualsQuery(database.perpetualDao(), database.searchDao())
    private val bitcoinAsset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8)
    private val ethereumAsset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val solanaAsset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)
    private val dogeAsset = mockAsset(id = mockAssetId(chain = Chain.Doge), name = "Dogecoin", symbol = "DOGE", decimals = 8)
    private val bitcoin = PerpetualData(
        perpetual = mockPerpetual(price = 95420.5, pricePercentChange24h = 2.5, volume24h = 500.0, funding = 0.0001).copy(id = PerpetualId(PerpetualProvider.Hypercore, "BTC"), name = "BTC", assetId = bitcoinAsset.id, identifier = "0"),
        asset = bitcoinAsset,
        metadata = PerpetualMetadata(isPinned = false),
    )
    private val ethereum = PerpetualData(
        perpetual = mockPerpetual(price = 3200.25, volume24h = 900.0).copy(id = PerpetualId(PerpetualProvider.Hypercore, "ETH"), name = "ETH", assetId = ethereumAsset.id, identifier = "1"),
        asset = ethereumAsset,
        metadata = PerpetualMetadata(isPinned = false),
    )
    private val solana = PerpetualData(
        perpetual = mockPerpetual(price = 150.0).copy(id = PerpetualId(PerpetualProvider.Hypercore, "SOL"), name = "SOL", assetId = solanaAsset.id, identifier = "2"),
        asset = solanaAsset,
        metadata = PerpetualMetadata(isPinned = true),
    )
    private val doge = PerpetualData(
        perpetual = mockPerpetual(price = 0.12).copy(id = PerpetualId(PerpetualProvider.Hypercore, "DOGE"), name = "DOGE", assetId = dogeAsset.id, identifier = "3"),
        asset = dogeAsset,
        metadata = PerpetualMetadata(isPinned = false),
    )

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        listOf(bitcoin, ethereum, solana, doge).forEach { database.assetsDao().insert(it.asset.toRecord()) }
        database.perpetualDao().insert(listOf(bitcoin, ethereum, solana, doge).map { it.perpetual.toDB(isPinned = it.metadata.isPinned) })
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun browsingListsPinnedMarketsFirstThenByVolumeWithTheirExactValues() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(solana, ethereum, bitcoin, doge), query("", 100, requiresVolume = false).first())
    }

    @Test
    fun browsingTradedMarketsKeepsPinnedOnesWithoutVolume() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(solana, ethereum, bitcoin), query("", 100, requiresVolume = true).first())
    }

    @Test
    fun theLimitCutsTheOrderedList() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(solana, ethereum), query("", 2, requiresVolume = false).first())
    }

    @Test
    fun aSearchWithoutStoredPrioritiesMatchesTheMarketNameOrTheAssetSymbol() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(ethereum, doge), query("e", 100, requiresVolume = true).first())
        assertEquals(listOf(solana, doge), query("o", 100, requiresVolume = true).first())
        assertEquals(emptyList<PerpetualData>(), query("bitcoin", 100, requiresVolume = true).first())
    }

    @Test
    fun aSearchWithStoredPrioritiesListsOnlyThoseMarketsInPriorityOrder() = runBlocking(Dispatchers.IO) {
        database.searchDao().insert(
            listOf(
                DbSearch(query = "bitcoin", perpetualId = doge.perpetual.id.toIdentifier(), priority = 0),
                DbSearch(query = "bitcoin", perpetualId = bitcoin.perpetual.id.toIdentifier(), priority = 1),
            ),
        )

        assertEquals(listOf(doge, bitcoin), query("bitcoin", 100, requiresVolume = true).first())
    }
}
