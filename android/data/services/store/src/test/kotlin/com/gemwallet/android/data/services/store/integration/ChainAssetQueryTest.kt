package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbAccount
import com.gemwallet.android.data.services.store.database.entities.DbAsset
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.AssetQueryOptional
import com.gemwallet.android.data.services.store.queries.ChainAssetQuery
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
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
class ChainAssetQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: ChainAssetQuery
    private val ethereum = AssetId(Chain.Ethereum)
    private val usdt = AssetId(Chain.Ethereum, "0xdAC17F958D2ee523a2206206994597C13D831ec7")
    private val usdc = AssetId(Chain.Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = ChainAssetQuery(AssetQueryOptional(database.assetsDao()))
        database.walletsDao().insert(DbWallet(id = "wallet-1", name = "wallet-1", domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        database.assetsDao().insert(
            listOf(
                DbAsset(id = "ethereum", chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18, type = AssetType.NATIVE),
                DbAsset(id = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", chain = Chain.Ethereum, name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20),
                DbAsset(id = "solana_EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", chain = Chain.Solana, name = "USD Coin", symbol = "USDC", decimals = 6, type = AssetType.SPL),
            ),
        )
        database.accountsDao().insert(listOf(DbAccount(walletId = "wallet-1", derivationPath = "m/44'/60'/0'/0/0", address = "0xabc", chain = Chain.Ethereum, extendedPublicKey = null)))
        database.balancesDao().insert(
            listOf(
                DbBalance(assetId = "ethereum", walletId = "wallet-1", available = "5", availableAmount = 5.0, isVisible = true, updatedAt = 0),
                DbBalance(assetId = "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", walletId = "wallet-1", available = "9", availableAmount = 9.0, isVisible = true, updatedAt = 0),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun aNativeAssetPaysItsOwnFee() = runBlocking(Dispatchers.IO) {
        val info = query("wallet-1", ethereum).first()

        assertEquals(ethereum, info?.assetInfo?.asset?.id)
        assertEquals(ethereum, info?.feeAssetInfo?.asset?.id)
    }

    @Test
    fun aTokenPaysItsFeeInTheChainCoin() = runBlocking(Dispatchers.IO) {
        val info = query("wallet-1", usdt).first()

        assertEquals(listOf(usdt, ethereum), listOf(info?.assetInfo?.asset?.id, info?.feeAssetInfo?.asset?.id))
        assertEquals(listOf(BigInteger.valueOf(9), BigInteger.valueOf(5)), listOf(info?.assetInfo?.balance?.balance?.available, info?.feeAssetInfo?.balance?.balance?.available))
    }

    @Test
    fun aTokenWhoseChainCoinIsMissingIsNull() = runBlocking(Dispatchers.IO) {
        assertNull(query("wallet-1", usdc).first())
    }
}
