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
import com.gemwallet.android.data.services.store.queries.NFTAssetQuery
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
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
class NFTAssetQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: NFTAssetQuery
    private val apes = NFTCollectionId(Chain.Ethereum, "0xapes")
    private val ape1 = NFTAssetId(Chain.Ethereum, "0xapes", "1")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = NFTAssetQuery(database.nftDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE))
        database.nftDao().add(collection = mockDbNftCollection(apes), asset = mockDbNftAsset(ape1, apes))
        database.nftDao().associateWithWallet(listOf(DbNFTAssociation(walletId = "wallet-1", assetId = ape1)))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theAssetCarriesItsCollection() = runBlocking(Dispatchers.IO) {
        val details = query("wallet-1", ape1).first()

        assertEquals(ape1, details?.assetData?.asset?.id)
        assertEquals(apes, details?.assetData?.collection?.id)
    }

    @Test
    fun theAssetIsOwnedOnlyByTheWalletHoldingIt() = runBlocking(Dispatchers.IO) {
        assertEquals(listOf(true, false), listOf(query("wallet-1", ape1).first()?.isOwned, query("wallet-2", ape1).first()?.isOwned))
    }

    @Test
    fun anAssetMissingFromTheStoreIsNull() = runBlocking(Dispatchers.IO) {
        assertNull(query("wallet-1", NFTAssetId(Chain.Ethereum, "0xapes", "2")).first())
    }
}
