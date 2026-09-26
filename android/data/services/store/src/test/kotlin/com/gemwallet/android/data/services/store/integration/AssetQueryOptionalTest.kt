package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAccount
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
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
import java.math.BigInteger

@RunWith(AndroidJUnit4::class)
class AssetQueryOptionalTest {
    private lateinit var database: GemDatabase
    private lateinit var query: AssetQueryOptional
    private val ethereum = AssetId(Chain.Ethereum)
    private val usdt = AssetId(Chain.Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = AssetQueryOptional(database.assetsDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE),
                DbAsset(id = "bitcoin", chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC", decimals = 8, type = AssetType.NATIVE),
                DbAsset(id = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", chain = Chain.Ethereum, name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20),
            ),
        )
        database.accountsDao().insert(
            listOf(
                DbAccount(walletId = "wallet-1", derivationPath = "m/44'/60'/0'/0/0", address = "0xabc", chain = Chain.Ethereum, extendedPublicKey = null),
                DbAccount(walletId = "wallet-2", derivationPath = "m/44'/60'/0'/0/0", address = "0xdef", chain = Chain.Ethereum, extendedPublicKey = null),
            ),
        )
        database.balancesDao().insert(
            listOf(
                DbBalance(assetId = "ethereum", walletId = "wallet-1", available = "5", availableAmount = 5.0, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "ethereum", walletId = "wallet-2", available = "7", availableAmount = 7.0, isVisible = true, updatedAt = 0),
            ),
        )
        database.pricesDao().insert(listOf(DbPrice(assetId = "ethereum", value = 2000.0, currency = Currency.USD)))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theAssetCarriesTheWalletAccountBalanceAndPrice() = runBlocking(Dispatchers.IO) {
        val first = query("wallet-1", ethereum).first()
        val second = query("wallet-2", ethereum).first()

        assertEquals(listOf("0xabc", "0xdef"), listOf(first?.account?.address, second?.account?.address))
        assertEquals(listOf(BigInteger.valueOf(5), BigInteger.valueOf(7)), listOf(first?.balance?.available, second?.balance?.available))
        assertEquals(2000.0, first?.price?.price)
    }

    @Test
    fun aTokenUsesTheAccountOfItsChain() = runBlocking(Dispatchers.IO) {
        val token = query("wallet-1", usdt).first()

        assertEquals(usdt, token?.asset?.id)
        assertEquals("0xabc", token?.account?.address)
        assertEquals(BigInteger.ZERO, token?.balance?.available)
    }

    @Test
    fun aChainWithoutAnAccountHasNoAddress() = runBlocking(Dispatchers.IO) {
        val bitcoin = query("wallet-1", AssetId(Chain.Bitcoin)).first()

        assertEquals(AssetId(Chain.Bitcoin), bitcoin?.asset?.id)
        assertEquals("", bitcoin?.account?.address)
    }

    @Test
    fun anAssetMissingFromTheStoreIsNull() = runBlocking(Dispatchers.IO) {
        assertNull(query("wallet-1", AssetId(Chain.Solana)).first())
    }
}
