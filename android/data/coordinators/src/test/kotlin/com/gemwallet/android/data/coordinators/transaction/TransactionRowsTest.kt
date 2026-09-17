package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockTransactionId
import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Test

class TransactionRowsTest {

    private val subject = TransactionRows()

    private val first = mockTransactionExtended(mockTransaction(id = mockTransactionId(hash = "first")))
    private val second = mockTransactionExtended(mockTransaction(id = mockTransactionId(hash = "second")))

    @Test
    fun unchangedTransactionsKeepTheirRow() {
        val items = listOf(first, second)

        val rows = subject.aggregates(items)

        assertEquals(rows, subject.aggregates(items))
        assertSame(rows.first(), subject.aggregates(items).first())
    }

    @Test
    fun changedTransactionGetsANewRow() {
        val pending = first.copy(transaction = first.transaction.copy(state = TransactionState.Pending))
        val rows = subject.aggregates(listOf(pending, second))

        val updated = subject.aggregates(listOf(first, second))

        assertNotSame(rows.first(), updated.first())
        assertEquals(TransactionState.Confirmed, updated.first().state)
        assertSame(rows.last(), updated.last())
    }

    @Test
    fun droppedTransactionIsNotKeptAlive() {
        subject.aggregates(listOf(first, second))

        val remaining = subject.aggregates(listOf(second))

        assertEquals(1, remaining.size)
    }
}
