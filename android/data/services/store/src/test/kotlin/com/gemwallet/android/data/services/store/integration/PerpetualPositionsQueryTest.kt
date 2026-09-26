package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.entities.toDB
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.PerpetualPositionsQuery
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualOrderType
import com.wallet.core.primitives.PerpetualPositionData
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PerpetualTriggerOrder
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
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
class PerpetualPositionsQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = PerpetualPositionsQuery(database.perpetualPositionDao())
    private val wallet1 = WalletId("wallet-1")
    private val wallet2 = WalletId("wallet-2")
    private val bitcoinAsset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "XBT", decimals = 8)
    private val ethereumAsset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val bitcoinMarket = mockPerpetual(price = 100.0).copy(id = PerpetualId(PerpetualProvider.Hypercore, "BTC-USD"), name = "BTC", assetId = bitcoinAsset.id, identifier = "0")
    private val ethereumMarket = mockPerpetual(price = 50.0).copy(id = PerpetualId(PerpetualProvider.Hypercore, "ETH"), name = "ETH", assetId = ethereumAsset.id, identifier = "1")
    private val bitcoinLong = PerpetualPositionData(
        perpetual = bitcoinMarket,
        asset = bitcoinAsset,
        position = mockPerpetualPosition(
            id = "btc-long",
            perpetualId = bitcoinMarket.id,
            assetId = bitcoinAsset.id,
            sizeValue = 200.0,
            entryPrice = 95420.5,
            liquidationPrice = 80000.25,
            marginAmount = 40.123456789,
            takeProfit = PerpetualTriggerOrder(price = 120000.0, order_type = PerpetualOrderType.Market, order_id = "tp-1"),
            pnl = -12.345678,
            funding = 0.5f,
        ).copy(size = 2.0),
    )
    private val ethereumShort = PerpetualPositionData(
        perpetual = ethereumMarket,
        asset = ethereumAsset,
        position = mockPerpetualPosition(id = "eth-short", perpetualId = ethereumMarket.id, assetId = ethereumAsset.id, direction = PerpetualDirection.Short).copy(size = -10.0),
    )
    private val otherWalletBitcoin = PerpetualPositionData(
        perpetual = bitcoinMarket,
        asset = bitcoinAsset,
        position = mockPerpetualPosition(id = "btc-other", perpetualId = bitcoinMarket.id, assetId = bitcoinAsset.id).copy(size = 1.0),
    )

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        listOf(wallet1, wallet2).forEach { id ->
            database.walletsDao().insert(DbWallet(id = id.id, name = id.id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        listOf(bitcoinAsset, ethereumAsset).forEach { database.assetsDao().insert(it.toRecord()) }
        database.perpetualDao().insert(listOf(bitcoinMarket, ethereumMarket).map { it.toDB() })
        database.perpetualPositionDao().upsertPositions(
            listOf(bitcoinLong.position.toDB(wallet1.id), ethereumShort.position.toDB(wallet1.id), otherWalletBitcoin.position.toDB(wallet2.id)),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theWalletPositionsComeByAbsoluteSizeTimesPriceDescendingWithTheirExactValues() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(ethereumShort, bitcoinLong), query(wallet1).first())
    }

    @Test
    fun anotherWalletSeesOnlyItsOwnPositions() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(otherWalletBitcoin), query(wallet2).first())
        assertEquals(emptyList<PerpetualPositionData>(), query(WalletId("wallet-3")).first())
    }

    @Test
    fun aSearchMatchesThePerpetualNameAndIdentifierAndTheAssetNameAndSymbol() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(bitcoinLong), query(wallet1, "btc").first())
        assertEquals(listOf(ethereumShort), query(wallet1, "1").first())
        assertEquals(listOf(ethereumShort), query(wallet1, "ether").first())
        assertEquals(listOf(bitcoinLong), query(wallet1, "xbt").first())
    }

    @Test
    fun aSearchDoesNotMatchThePerpetualIdSymbol() = runBlocking(Dispatchers.IO) {
        assertEquals(emptyList<PerpetualPositionData>(), query(wallet1, "btc-usd").first())
    }

    @Test
    fun thePositionIsFoundByWalletAndMarket() = runBlocking(Dispatchers.IO) {
        assertEquals(bitcoinLong, query(wallet1, bitcoinMarket.id).first())
        assertEquals(otherWalletBitcoin, query(wallet2, bitcoinMarket.id).first())
    }

    @Test
    fun aMarketWithoutAPositionInTheWalletHasNone() = runBlocking(Dispatchers.IO) {
        assertNull(query(wallet2, ethereumMarket.id).first())
    }
}
