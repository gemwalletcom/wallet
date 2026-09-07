package com.gemwallet.android.service.store

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbTransaction
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.TransactionState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TransactionIdentityTest {
    private lateinit var database: GemDatabase
    private lateinit var transactions: TransactionsDao
    private val wallet = mockWallet()
    private val pending = mockTransaction(state = TransactionState.Pending).toRecord(wallet.id)
    private val finalId = mockTransactionId(hash = "final-hash")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        transactions = database.transactionsDao()
        database.walletsDao().insert(wallet.toRecord())
        database.assetsDao().insert(mockAsset().toRecord())
    }

    @After
    fun tearDown() {
        database.close()
    }

    @Test
    fun observationFollowsInsertHashUpdateAndRepeatedSync() = runBlocking(Dispatchers.IO) {
        transactions.insert(listOf(mockTransaction(id = mockTransactionId(hash = "unrelated")).toRecord(wallet.id)))
        withTimeout(10_000) {
            val updates = Channel<DbTransaction?>(Channel.UNLIMITED)
            val observer = launch {
                transactions.getExtendedTransaction(wallet.id, pending.id)
                    .map { it?.transaction }.distinctUntilChanged().collect { updates.send(it) }
            }
            try {
                assertNull(updates.receive())
                transactions.insert(listOf(pending))
                val inserted = requireNotNull(updates.receive())
                assertTrue(inserted.recordId > 0)
                assertEquals(inserted.recordId, transactions.getExtendedTransaction(wallet.id, pending.id).first()?.toDTO()?.recordId)

                transactions.updateTransactionHash(pending.id, wallet.id, "final-hash")
                val updated = requireNotNull(updates.receive())
                assertEquals(inserted.recordId, updated.recordId)
                assertEquals(finalId, updated.id)
                assertEquals("final-hash", updated.hash)

                transactions.insert(listOf(updated.copy(recordId = 0, state = TransactionState.Confirmed, fee = "250")))
                val confirmed = requireNotNull(updates.receive())
                assertEquals(inserted.recordId, confirmed.recordId)
                assertEquals(TransactionState.Confirmed, confirmed.state)
                assertEquals("250", confirmed.fee)

                transactions.delete(finalId, wallet.id)
                assertNull(updates.receive())
                transactions.insert(listOf(pending))
                assertTrue(requireNotNull(transactions.getTransaction(pending.id, wallet.id)).recordId > inserted.recordId)
            } finally {
                observer.cancel()
                updates.close()
            }
        }
    }

    @Test
    fun mergePreservesObservedIdentityAndConfirmedDataWithinWallet() = runBlocking(Dispatchers.IO) {
        val secondWallet = mockWallet(id = "wallet-2")
        database.walletsDao().insert(secondWallet.toRecord())
        val target = pending.copy(id = finalId, hash = "final-hash", state = TransactionState.Confirmed, fee = "300", metadata = "confirmed", value = "900")
        transactions.insert(listOf(pending, target, pending.copy(walletId = secondWallet.id), target.copy(walletId = secondWallet.id)))
        val source = requireNotNull(transactions.getTransaction(pending.id, wallet.id))
        val secondSource = transactions.getTransaction(pending.id, secondWallet.id)
        val secondTarget = transactions.getTransaction(finalId, secondWallet.id)
        assertNotEquals(source.recordId, transactions.getTransaction(finalId, wallet.id)?.recordId)

        withTimeout(10_000) {
            val updates = Channel<DbTransaction?>(Channel.UNLIMITED)
            val observer = launch {
                transactions.getExtendedTransaction(wallet.id, pending.id)
                    .map { it?.transaction }.distinctUntilChanged().collect { updates.send(it) }
            }
            try {
                assertEquals(source, updates.receive())
                transactions.updateTransactionHash(pending.id, wallet.id, "final-hash", updatedAt = 50)
                assertEquals(target.copy(recordId = source.recordId, updatedAt = 50), updates.receive())
                assertNull(transactions.getTransaction(pending.id, wallet.id))
                assertEquals(secondSource, transactions.getTransaction(pending.id, secondWallet.id))
                assertEquals(secondTarget, transactions.getTransaction(finalId, secondWallet.id))
                assertEquals(1, transactions.getExtendedTransactions(wallet.id).first().size)
            } finally {
                observer.cancel()
                updates.close()
            }
        }
    }

    @Test
    fun sameIdAndMissingSourceLeaveRecordsUnchanged() = runBlocking(Dispatchers.IO) {
        transactions.insert(listOf(pending))
        val source = transactions.getTransaction(pending.id, wallet.id)
        transactions.updateTransactionHash(pending.id, wallet.id, pending.id.hash, updatedAt = 50)
        transactions.updateTransactionHash(finalId, wallet.id, pending.id.hash, updatedAt = 60)
        assertEquals(source, transactions.getTransaction(pending.id, wallet.id))
    }
}
