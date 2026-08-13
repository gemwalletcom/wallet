package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.blockchain.services.TransactionStatusService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.DbAssetProjection
import com.gemwallet.android.data.service.store.database.entities.DbTransactionExtended
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Transaction
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class TransactionsRepositoryImplTest {

    private val transactionsDao = mockk<TransactionsDao>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository> {
        every { session() } returns MutableStateFlow<Session?>(null)
    }

    private val subject = TransactionsRepositoryImpl(
        sessionRepository = sessionRepository,
        transactionsDao = transactionsDao,
        transactionStatusService = mockk<TransactionStatusService>(),
    )

    @Test
    fun storeTransactionUpdate_rowRemoved_terminatesWithoutInsert() = runBlocking {
        every { transactionsDao.updateTransaction(any(), any(), any(), any(), any(), any(), any()) } returns 0

        val result = subject.storeTransactionUpdate(
            currentTransaction = extended(mockTransaction(state = TransactionState.Pending)),
            updatedTransaction = extended(mockTransaction(state = TransactionState.Confirmed)),
        )

        assertNull(result)
        verify(exactly = 0) { transactionsDao.insert(any()) }
    }

    @Test
    fun storeTransactionUpdate_rowPresent_storesUpdate() = runBlocking {
        every { transactionsDao.updateTransaction(any(), any(), any(), any(), any(), any(), any()) } returns 1

        val result = subject.storeTransactionUpdate(
            currentTransaction = extended(mockTransaction(state = TransactionState.Pending)),
            updatedTransaction = extended(mockTransaction(state = TransactionState.Confirmed)),
        )

        assertEquals(TransactionState.Confirmed, result?.transaction?.state)
        verify(exactly = 0) { transactionsDao.insert(any()) }
    }

    @Test
    fun publishEnteringInTransit_emitsChange() {
        val current = mockTransaction(state = TransactionState.Pending)
        val updated = current.copy(state = TransactionState.InTransit)

        subject.publishEnteringInTransit(extended(current), extended(updated))

        assertEquals(
            listOf(TransactionState.InTransit),
            subject.changedTransactions.value.map { it.transaction.state },
        )
    }

    @Test
    fun publishEnteringInTransit_ignoresOtherTransitions() {
        val current = mockTransaction(state = TransactionState.Pending)
        val updated = current.copy(state = TransactionState.Confirmed)

        subject.publishEnteringInTransit(extended(current), extended(updated))

        assertEquals(emptyList<TransactionState>(), subject.changedTransactions.value.map { it.transaction.state })
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
            currentTransaction = extended(current),
            updatedTransaction = extended(updated),
        )

        assertNull(result)
        verify(exactly = 0) { transactionsDao.insert(any()) }
    }

    private fun extended(transaction: Transaction) = DbTransactionExtended(
        transaction = transaction.toRecord(WalletId("wallet-1")),
        asset = assetProjection(),
        feeAsset = assetProjection(),
        priceValue = null,
        priceDayChanged = null,
        feePriceValue = null,
        feePriceDayChanged = null,
        fromAsset = null,
        toAsset = null,
        fromAddress = null,
        toAddress = null,
    )

    private fun assetProjection() = DbAssetProjection(
        id = "bitcoin",
        name = "Asset",
        symbol = "A",
        decimals = 8,
        type = AssetType.NATIVE,
    )
}
