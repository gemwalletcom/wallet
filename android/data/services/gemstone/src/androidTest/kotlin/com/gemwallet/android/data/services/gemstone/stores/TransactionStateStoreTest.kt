package com.gemwallet.android.data.services.gemstone.stores

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.service.store.database.GemDatabase
import com.gemwallet.android.data.service.store.database.RoomStoreTransactionRunner
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TransactionStateStoreTest {
    @Test
    fun hashUpdateKeepsSwapMetadataForOtherWalletUntilLastReferenceIsMoved() = runBlocking(Dispatchers.IO) {
        val database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        try {
            val runner = RoomStoreTransactionRunner(database)
            val wallets = GemstoneWalletStore(database.walletsDao(), database.accountsDao(), database.assetsDao(), runner)
            val store = GemstoneTransactionStateStore(database.transactionsDao(), wallets, runner)
            val wallet = mockWallet()
            val otherWallet = mockWallet(id = "wallet-2")
            database.walletsDao().insert(wallet.toRecord())
            database.walletsDao().insert(otherWallet.toRecord())
            val metadata = TransactionSwapMetadata(mockAssetId(), "100", mockAssetId(chain = Chain.Ethereum), "200")
            val pending = mockTransaction(type = TransactionType.Swap, state = TransactionState.Pending, metadata = metadata.toJson())
            val confirmed = pending.copy(id = mockTransactionId(hash = "final-hash"), state = TransactionState.Confirmed, metadata = metadata.copy(toValue = "250").toJson())
            store.addTransactions(wallet.id.id, listOf(pending.toJson(), confirmed.toJson()))
            store.addTransactions(otherWallet.id.id, listOf(pending.toJson()))
            val recordId = database.transactionsDao().getTransaction(pending.id, wallet.id)?.recordId

            store.updateTransactionHash(wallet.id.id, pending.id.identifier, confirmed.id.hash)
            assertEquals(recordId, database.transactionsDao().getTransaction(confirmed.id, wallet.id)?.recordId)
            database.openHelper.readableDatabase.query("SELECT tx_id, to_amount FROM tx_swap_metadata ORDER BY tx_id").use { cursor ->
                assertEquals(2, cursor.count)
                cursor.moveToFirst()
                assertEquals(confirmed.id.identifier, cursor.getString(0))
                assertEquals("250", cursor.getString(1))
                cursor.moveToNext()
                assertEquals(pending.id.identifier, cursor.getString(0))
                assertEquals("200", cursor.getString(1))
            }

            store.updateTransactionHash(otherWallet.id.id, pending.id.identifier, confirmed.id.hash)
            database.openHelper.readableDatabase.query("SELECT tx_id, to_amount FROM tx_swap_metadata").use { cursor ->
                assertEquals(1, cursor.count)
                cursor.moveToFirst()
                assertEquals(confirmed.id.identifier, cursor.getString(0))
                assertEquals("250", cursor.getString(1))
            }
            val updatedId = mockTransactionId(hash = "updated-swap")
            val swap = pending.copy(id = mockTransactionId(hash = "swap-hash"))
            store.addTransactions(wallet.id.id, listOf(swap.toJson()))
            store.updateTransactionHash(wallet.id.id, swap.id.identifier, updatedId.hash)
            database.openHelper.readableDatabase.query("SELECT tx_id, to_amount FROM tx_swap_metadata WHERE tx_id IN (?, ?)", arrayOf(swap.id.identifier, updatedId.identifier)).use { cursor ->
                assertEquals(1, cursor.count)
                cursor.moveToFirst()
                assertEquals(updatedId.identifier, cursor.getString(0))
                assertEquals("200", cursor.getString(1))
            }

            val swapForTransfer = swap.copy(id = mockTransactionId(hash = "swap-for-transfer"))
            val transfer = confirmed.copy(id = mockTransactionId(hash = "transfer-hash"), type = TransactionType.Transfer, metadata = null)
            store.addTransactions(wallet.id.id, listOf(swapForTransfer.toJson(), transfer.toJson()))
            store.updateTransactionHash(wallet.id.id, swapForTransfer.id.identifier, transfer.id.hash)
            database.openHelper.readableDatabase.query("SELECT tx_id FROM tx_swap_metadata WHERE tx_id IN (?, ?)", arrayOf(swapForTransfer.id.identifier, transfer.id.identifier)).use { cursor ->
                assertEquals(0, cursor.count)
            }
        } finally {
            database.close()
        }
    }
}
