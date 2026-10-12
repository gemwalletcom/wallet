package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.queries.AssociatedAssetsQuery
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetAssociationType
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AssociatedAssetsQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: AssociatedAssetsQuery
    private val perpetual = AssetId(Chain.HyperCore, "perpetual::BTC")
    private val bitcoin = AssetId(Chain.Bitcoin)
    private val wrapped = AssetId(Chain.Ethereum, "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = AssociatedAssetsQuery(database.assetsDao())
        database.assetsDao().insert(
            listOf(
                DbAsset(
                    id = perpetual.toIdentifier(),
                    chain = Chain.HyperCore,
                    name = "BTC",
                    symbol = "BTC",
                    decimals = 5,
                    type = AssetType.PERPETUAL,
                    associations = listOf(AssetAssociation(bitcoin, AssetAssociationType.Official), AssetAssociation(wrapped, AssetAssociationType.Wrapped)),
                ),
                DbAsset(id = bitcoin.toIdentifier(), chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE),
                DbAsset(id = wrapped.toIdentifier(), chain = Chain.Ethereum, name = "Wrapped BTC", symbol = "WBTC", decimals = 8, type = AssetType.ERC20),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun assetsOfType() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(bitcoin), query(perpetual, AssetAssociationType.Official).first().map { it.id })
        assertEquals(emptyList<AssetId>(), query(bitcoin, AssetAssociationType.Official).first().map { it.id })
    }
}
