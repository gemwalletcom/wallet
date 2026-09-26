package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbNFTAssociation
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.entities.mockDbNftAsset
import com.gemwallet.android.data.services.store.database.entities.mockDbNftCollection
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NFTQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: NFTQuery
    private val apes = NFTCollectionId(Chain.Ethereum, "0xapes")
    private val punks = NFTCollectionId(Chain.Ethereum, "0xpunks")
    private val ape1 = NFTAssetId(Chain.Ethereum, "0xapes", "1")
    private val ape2 = NFTAssetId(Chain.Ethereum, "0xapes", "2")
    private val punk1 = NFTAssetId(Chain.Ethereum, "0xpunks", "1")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = NFTQuery(database.nftDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE))
        database.nftDao().insertCollections(listOf(mockDbNftCollection(apes), mockDbNftCollection(punks)))
        database.nftDao().insertAssets(listOf(mockDbNftAsset(ape1, apes), mockDbNftAsset(ape2, apes), mockDbNftAsset(punk1, punks)))
        database.nftDao().associateWithWallet(
            listOf(
                DbNFTAssociation(walletId = "wallet-1", assetId = ape1),
                DbNFTAssociation(walletId = "wallet-1", assetId = ape2),
                DbNFTAssociation(walletId = "wallet-2", assetId = punk1),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun listsTheWalletCollectionsWithTheirWalletAssets() = runBlocking(Dispatchers.IO) {
        val first = query("wallet-1").first()
        val second = query("wallet-2").first()

        assertEquals(listOf(apes to setOf(ape1, ape2)), first.map { it.collection.id to it.assets.map { asset -> asset.id }.toSet() })
        assertEquals(listOf(punks to listOf(punk1)), second.map { it.collection.id to it.assets.map { asset -> asset.id } })
    }

    @Test
    fun aCollectionIdentifierKeepsOnlyThatCollection() = runBlocking(Dispatchers.IO) {
        database.nftDao().associateWithWallet(listOf(DbNFTAssociation(walletId = "wallet-1", assetId = punk1)))

        assertEquals(setOf(apes, punks), query("wallet-1").first().map { it.collection.id }.toSet())
        assertEquals(listOf(punks), query("wallet-1", "ethereum_0xpunks").first().map { it.collection.id })
        assertEquals(emptyList<NFTData>(), query("wallet-1", "ethereum_0xmissing").first())
    }

    @Test
    fun aWalletWithoutAssetsListsNothing() = runBlocking(Dispatchers.IO) {
        database.walletsDao().insert(DbWallet(id = "wallet-3", name = "wallet-3", domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))

        assertEquals(emptyList<NFTData>(), query("wallet-3").first())
    }
}
