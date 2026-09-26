package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbPrice
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.TransactionQuery
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Price
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
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
class TransactionQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = TransactionQuery(database.transactionsDao())
    private val wallet = mockWallet(id = WalletId("wallet-1"))
    private val otherWallet = mockWallet(id = WalletId("wallet-2"))
    private val ethereum = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val usdt = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xdac17f958d2ee523a2206206994597c13d831ec7"), name = "Tether", symbol = "USDT", decimals = 6, type = AssetType.ERC20)
    private val send = mockTransaction(
        assetId = usdt.id,
        id = mockTransactionId(chain = Chain.Ethereum, hash = "0xsend"),
        from = "0xsender",
        to = "0xrecipient",
        state = TransactionState.Pending,
        feeAssetId = ethereum.id,
        value = "123456789012345678901234567890",
        createdAt = 100,
    )
    private val otherWalletSend = mockTransaction(assetId = ethereum.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "0xother"), value = "5", createdAt = 200, state = TransactionState.Confirmed, feeAssetId = ethereum.id)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.walletsDao().insert(wallet.toRecord())
        database.walletsDao().insert(otherWallet.toRecord())
        database.assetsDao().insert(listOf(ethereum, usdt).map { it.toRecord() })
        database.pricesDao().insert(DbPrice(assetId = usdt.id.toIdentifier(), value = 1.001, dayChanged = -0.2, currency = Currency.USD))
        database.pricesDao().insert(DbPrice(assetId = ethereum.id.toIdentifier(), value = 2500.5, dayChanged = 3.1, currency = Currency.USD))
        database.transactionsDao().insert(listOf(send.toRecord(wallet.id), otherWalletSend.toRecord(otherWallet.id)))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theTransactionIsFoundByWalletAndIdWithItsAssetsAndPrices() = runBlocking(Dispatchers.IO) {
        val transaction = query(wallet.id, send.id).first()

        assertEquals(send.stored(), transaction?.transaction)
        assertEquals(usdt, transaction?.asset)
        assertEquals(ethereum, transaction?.feeAsset)
        assertEquals(Price(1.001, -0.2, 0L), transaction?.price)
        assertEquals(Price(2500.5, 3.1, 0L), transaction?.feePrice)
    }

    @Test
    fun aTransactionOfAnotherWalletOrAnUnknownIdIsNotFound() = runBlocking(Dispatchers.IO) {
        assertNull(query(otherWallet.id, send.id).first())
        assertNull(query(wallet.id, otherWalletSend.id).first())
        assertNull(query(wallet.id, mockTransactionId(chain = Chain.Ethereum, hash = "0xmissing")).first())
    }

    @Test
    fun aStateChangeIsReadBack() = runBlocking(Dispatchers.IO) {
        database.transactionsDao().updateTransactionState(send.id, wallet.id, TransactionState.Confirmed, fee = "21000", blockNumber = "19000000", metadata = null, confirmationEtaSeconds = null)

        assertEquals(send.stored().copy(state = TransactionState.Confirmed, fee = "21000", blockNumber = "19000000"), query(wallet.id, send.id).first()?.transaction)
    }

    @Test
    fun aDeletedTransactionIsNoLongerFound() = runBlocking(Dispatchers.IO) {
        database.transactionsDao().delete(send.id, wallet.id)

        assertNull(query(wallet.id, send.id).first())
    }

    private fun Transaction.stored(): Transaction = copy(blockNumber = blockNumber.orEmpty(), sequence = "", utxoInputs = emptyList(), utxoOutputs = emptyList())
}
