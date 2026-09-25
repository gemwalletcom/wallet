package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.application.transactions.values.TransactionsQueryFilter
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.TransactionsCountQuery
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TransactionsCountQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = TransactionsCountQuery(database.transactionsDao())
    private val pending = TransactionsQueryFilter.pendingActivity()
    private val wallet = mockWallet(id = "wallet-1")
    private val otherWallet = mockWallet(id = "wallet-2")
    private val bitcoin = mockAsset(id = mockAssetId(chain = Chain.Bitcoin), name = "Bitcoin", symbol = "BTC", decimals = 8)
    private val ethereum = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val spam = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0xspam"), name = "Spam", symbol = "SPAM", decimals = 18, type = AssetType.ERC20)
    private val sending = mockTransaction(assetId = bitcoin.id, id = mockTransactionId(chain = Chain.Bitcoin, hash = "sending"), state = TransactionState.Pending)
    private val bridging = mockTransaction(assetId = ethereum.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "bridging"), type = TransactionType.Swap, state = TransactionState.InTransit)
    private val confirmed = mockTransaction(assetId = ethereum.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "confirmed"))
    private val failed = mockTransaction(assetId = ethereum.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "failed"), state = TransactionState.Failed)
    private val spamPending = mockTransaction(assetId = spam.id, id = mockTransactionId(chain = Chain.Ethereum, hash = "spam"), feeAssetId = ethereum.id, state = TransactionState.Pending)
    private val otherWalletPending = mockTransaction(assetId = bitcoin.id, id = mockTransactionId(chain = Chain.Bitcoin, hash = "other"), state = TransactionState.Pending)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.walletsDao().insert(wallet.toRecord())
        database.walletsDao().insert(otherWallet.toRecord())
        database.assetsDao().insert(listOf(bitcoin.toRecord().copy(rank = 100), ethereum.toRecord().copy(rank = 90), spam.toRecord().copy(rank = 5)))
        database.transactionsDao().insert(listOf(sending, bridging, confirmed, failed, spamPending).toRecord(wallet.id) + listOf(otherWalletPending).toRecord(otherWallet.id))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun thePendingCountHoldsTheWalletUnfinishedActivityOnly() = runBlocking(Dispatchers.IO) {
        assertEquals(2, query(wallet.id, pending).first())
        assertEquals(1, query(otherWallet.id, pending).first())
    }

    @Test
    fun withoutFiltersEveryWalletTransactionIsCounted() = runBlocking(Dispatchers.IO) {
        assertEquals(5, query(wallet.id, emptyList()).first())
    }

    @Test
    fun theCountFollowsATransactionThatFinishes() = runBlocking(Dispatchers.IO) {
        val counts = query(wallet.id, pending)
        assertEquals(2, counts.first())

        database.transactionsDao().updateTransactionState(sending.id, wallet.id, TransactionState.Confirmed, fee = null, blockNumber = null, metadata = null, confirmationEtaSeconds = null)

        assertEquals(1, counts.first())
    }

    @Test
    fun aWalletWithoutPendingTransactionsCountsZero() = runBlocking(Dispatchers.IO) {
        database.walletsDao().insert(mockWallet(id = "wallet-3").toRecord())

        assertEquals(0, query(mockWallet(id = "wallet-3").id, pending).first())
    }
}
