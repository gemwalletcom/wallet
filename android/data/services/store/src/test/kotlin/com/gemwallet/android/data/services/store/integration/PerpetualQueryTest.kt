package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.PerpetualQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockPerpetual
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
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
    private val query = PerpetualQuery(database.perpetualDao(), database.assetsDao())
    private val bitcoinAsset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8u)
    private val ethereumAsset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18u)
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
        database.perpetualDao().insert(listOf(bitcoin, ethereum).map { it.perpetual.toRecord().copy(isPinned = it.metadata.isPinned) })
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
    fun theMarketListsItsOfficialAssociatedAssets() = runBlocking(Dispatchers.IO) {
        val perpetualAssetId = AssetId(Chain.HyperCore, "perpetual::CAKE")
        val wrapped = AssetId(Chain.Ethereum, "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599")
        database.assetsDao().insert(
            listOf(
                DbAsset(
                    id = perpetualAssetId.toIdentifier(),
                    chain = Chain.HyperCore,
                    name = "CAKE",
                    symbol = "CAKE",
                    decimals = 1,
                    type = AssetType.PERPETUAL,
                    associations = listOf(AssetAssociation(ethereumAsset.id, AssetAssociationType.Official), AssetAssociation(wrapped, AssetAssociationType.Wrapped)),
                ),
                DbAsset(id = wrapped.toIdentifier(), chain = Chain.Ethereum, name = "Wrapped BTC", symbol = "WBTC", decimals = 8, type = AssetType.ERC20),
            ),
        )
        database.perpetualDao().insert(listOf(mockPerpetual().copy(id = PerpetualId(PerpetualProvider.Hypercore, "CAKE"), name = "CAKE", assetId = perpetualAssetId, identifier = "2").toRecord()))

        assertEquals(listOf(ethereumAsset.id), query.associatedAssets(perpetualAssetId).first().map { it.id })
        assertEquals(emptyList<AssetId>(), query.associatedAssets(ethereumAsset.id).first().map { it.id })
    }

    @Test
    fun anUnknownAssetOrIdHasNoMarket() = runBlocking(Dispatchers.IO) {
        assertNull(query(mockAssetId(chain = Chain.Solana)).first())
        assertNull(query(PerpetualId(PerpetualProvider.Hypercore, "SOL")).first())
    }
}
