package com.gemwallet.android.data.services.gemstone.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.RoomStoreTransactionRunner
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStateStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemTransactionStateUpdate

@RunWith(AndroidJUnit4::class)
class TransactionStateStoreTest {
    @Test
    fun hashUpdateKeepsTransactionAssetsForOtherWalletUntilLastReferenceIsMoved() = runBlocking(Dispatchers.IO) {
        val database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        try {
            val runner = RoomStoreTransactionRunner(database)
            val store = GemstoneTransactionStateStore(database.transactionsDao(), runner)
            val wallet = mockWallet()
            val otherWallet = mockWallet(id = "wallet-2")
            database.walletsDao().insert(wallet.toRecord())
            database.walletsDao().insert(otherWallet.toRecord())
            val metadata = TransactionSwapMetadata(mockAssetId(), "100", mockAssetId(chain = Chain.Ethereum), "200")
            val swapAssets = listOf(metadata.fromAsset.toIdentifier(), metadata.toAsset.toIdentifier()).sorted()
            val pending = mockTransaction(type = TransactionType.Swap, state = TransactionState.Pending, metadata = metadata.toJson())
            val confirmed = pending.copy(id = mockTransactionId(hash = "final-hash"), state = TransactionState.Confirmed, metadata = metadata.copy(toValue = "250").toJson())
            store.addTransactions(wallet.id.id, listOf(pending.toGem(), confirmed.toGem()))
            store.addTransactions(otherWallet.id.id, listOf(pending.toGem()))
            val recordId = database.transactionsDao().getTransaction(pending.id, wallet.id)?.recordId

            store.updateTransactionHash(wallet.id.id, pending.id.identifier, confirmed.id.hash)
            assertEquals(recordId, database.transactionsDao().getTransaction(confirmed.id, wallet.id)?.recordId)
            assertEquals(mapOf(confirmed.id.identifier to swapAssets, pending.id.identifier to swapAssets), database.transactionAssets(pending.id, confirmed.id))

            store.updateTransactionHash(otherWallet.id.id, pending.id.identifier, confirmed.id.hash)
            assertEquals(mapOf(confirmed.id.identifier to swapAssets), database.transactionAssets(pending.id, confirmed.id))

            val updatedId = mockTransactionId(hash = "updated-swap")
            val swap = pending.copy(id = mockTransactionId(hash = "swap-hash"))
            store.addTransactions(wallet.id.id, listOf(swap.toGem()))
            store.updateTransactionHash(wallet.id.id, swap.id.identifier, updatedId.hash)
            assertEquals(mapOf(updatedId.identifier to swapAssets), database.transactionAssets(swap.id, updatedId))

            val swapForTransfer = swap.copy(id = mockTransactionId(hash = "swap-for-transfer"))
            val transfer = confirmed.copy(id = mockTransactionId(hash = "transfer-hash"), type = TransactionType.Transfer, metadata = null)
            store.addTransactions(wallet.id.id, listOf(swapForTransfer.toGem(), transfer.toGem()))
            store.updateTransactionHash(wallet.id.id, swapForTransfer.id.identifier, transfer.id.hash)
            assertEquals(mapOf(transfer.id.identifier to listOf(transfer.assetId.toIdentifier())), database.transactionAssets(swapForTransfer.id, transfer.id))
        } finally {
            database.close()
        }
    }

    @Test
    fun stateUpdateReplacesTheTransactionAssetsCoreReports() = runBlocking(Dispatchers.IO) {
        val database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        try {
            val store = GemstoneTransactionStateStore(database.transactionsDao(), RoomStoreTransactionRunner(database))
            val wallet = mockWallet()
            database.walletsDao().insert(wallet.toRecord())
            val pending = mockTransaction(type = TransactionType.Swap, state = TransactionState.Pending)
            store.addTransactions(wallet.id.id, listOf(pending.toGem()))
            val assetIds = listOf(mockAssetId().toIdentifier(), mockAssetId(chain = Chain.Ethereum).toIdentifier())

            store.updateTransaction(wallet.id.id, pending.id.identifier, GemTransactionStateUpdate(TransactionState.Confirmed.toGem(), null, null, null, null, assetIds))

            assertEquals(mapOf(pending.id.identifier to assetIds.sorted()), database.transactionAssets(pending.id))
        } finally {
            database.close()
        }
    }

    private fun GemDatabase.transactionAssets(vararg ids: TransactionId): Map<String, List<String>> = openHelper.readableDatabase.query(
        "SELECT tx_id, asset_id FROM transactions_assets WHERE tx_id IN (${ids.joinToString { "?" }}) ORDER BY tx_id, asset_id",
        ids.map { it.identifier }.toTypedArray(),
    ).use { cursor ->
        buildList { while (cursor.moveToNext()) add(cursor.getString(0) to cursor.getString(1)) }
    }.groupBy({ it.first }, { it.second })
}
