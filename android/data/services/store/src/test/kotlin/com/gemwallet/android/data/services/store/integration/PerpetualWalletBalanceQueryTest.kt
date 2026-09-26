package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalance
import com.gemwallet.android.data.services.store.queries.PerpetualWalletBalanceQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualBalance
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
class PerpetualWalletBalanceQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = PerpetualWalletBalanceQuery(database.balancesDao(), database.pricesDao())
    private val wallet1 = WalletId("wallet-1")
    private val wallet2 = WalletId("wallet-2")
    private val collateral = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "USDC"), name = "USDC", symbol = "USDC", decimals = 8, type = AssetType.PERPETUAL)
    private val bitcoin = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        listOf(wallet1, wallet2).forEach { id ->
            database.walletsDao().insert(DbWallet(id = id.id, name = id.id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        listOf(collateral, bitcoin).forEach { database.assetsDao().insert(it.toRecord()) }
        database.balancesDao().insert(
            listOf(
                DbBalance(assetId = collateral.id.toIdentifier(), walletId = wallet1.id, availableAmount = 1234.56789012, reservedAmount = 0.00000001, withdrawableAmount = 987.654321, updatedAt = 0),
                DbBalance(assetId = bitcoin.id.toIdentifier(), walletId = wallet2.id, availableAmount = 5.0, reservedAmount = 1.0, withdrawableAmount = 4.0, updatedAt = 0),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theWalletCollateralCarriesItsExactAmountsAndPrice() = runBlocking(Dispatchers.IO) {
        database.pricesDao().insert(DbPrice(assetId = collateral.id.toIdentifier(), value = 0.99987654, currency = Currency.USD))

        assertEquals(
            PerpetualWalletBalance(balance = PerpetualBalance(available = 1234.56789012, reserved = 0.00000001, withdrawable = 987.654321), price = 0.99987654),
            query(wallet1, collateral.id).first(),
        )
    }

    @Test
    fun aCollateralWithoutAStoredPriceIsPricedAtZero() = runBlocking(Dispatchers.IO) {
        assertEquals(0.0, query(wallet1, collateral.id).first()?.price)
    }

    @Test
    fun aWalletWithoutTheCollateralBalanceHasNone() = runBlocking(Dispatchers.IO) {
        assertNull(query(wallet2, collateral.id).first())
        assertNull(query(wallet1, bitcoin.id).first())
    }
}
