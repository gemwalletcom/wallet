package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.mockDbTransactionExtended
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.wallet.core.primitives.TransactionState
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class TransactionStateServiceTest {

    private val transactionsDao = mockk<TransactionsDao>(relaxed = true)

    private val subject = TransactionStateService(
        transactionsDao = transactionsDao,
        transactionStatusService = mockk<TransactionStatusService>(),
    )

    @Test
    fun storeTransactionUpdate_rowRemoved_terminatesWithoutInsert() = runBlocking {
        every { transactionsDao.updateTransaction(any(), any(), any(), any(), any(), any(), any()) } returns 0

        val result = subject.storeTransactionUpdate(
            currentTransaction = mockDbTransactionExtended(mockTransaction(state = TransactionState.Pending)),
            updatedTransaction = mockDbTransactionExtended(mockTransaction(state = TransactionState.Confirmed)),
        )

        assertNull(result)
        verify(exactly = 0) { transactionsDao.insert(any()) }
    }

    @Test
    fun storeTransactionUpdate_rowPresent_storesUpdate() = runBlocking {
        every { transactionsDao.updateTransaction(any(), any(), any(), any(), any(), any(), any()) } returns 1

        val result = subject.storeTransactionUpdate(
            currentTransaction = mockDbTransactionExtended(mockTransaction(state = TransactionState.Pending)),
            updatedTransaction = mockDbTransactionExtended(mockTransaction(state = TransactionState.Confirmed)),
        )

        assertEquals(TransactionState.Confirmed, result?.transaction?.state)
        verify(exactly = 0) { transactionsDao.insert(any()) }
    }

    @Test
    fun storeTransactionUpdate_hashChangedAndRowRemoved_terminatesWithoutInsert() = runBlocking {
        every { transactionsDao.getTransactionState(any(), any()) } returns null
        every { transactionsDao.updateTransaction(any(), any(), any(), any(), any(), any(), any()) } returns 0
        val current = mockTransaction(state = TransactionState.Pending)
        val updated = current.copy(
            id = mockTransactionId(hash = "replaced-tx-id"),
            state = TransactionState.Confirmed,
        )

        val result = subject.storeTransactionUpdate(
            currentTransaction = mockDbTransactionExtended(current),
            updatedTransaction = mockDbTransactionExtended(updated),
        )

        assertNull(result)
        verify(exactly = 0) { transactionsDao.insert(any()) }
    }
}
