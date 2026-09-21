package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockTransactionId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Test

class TransactionRowsTest {

    private val subject = TransactionRows()
    private val activity = TransactionsRequestFilter.activityDefaults()
    private val asset = listOf(TransactionsRequestFilter.Chains(listOf(Chain.Bitcoin)))

    private val first = mockTransactionExtended(mockTransaction(id = mockTransactionId(hash = "first")))
    private val second = mockTransactionExtended(mockTransaction(id = mockTransactionId(hash = "second")))

    @Test
    fun unchangedTransactionsKeepTheirRow() {
        val items = listOf(first, second)

        val rows = subject.aggregates(activity, items)

        assertEquals(rows, subject.aggregates(activity, items))
        assertSame(rows.first(), subject.aggregates(activity, items).first())
    }

    @Test
    fun changedTransactionGetsANewRow() {
        val pending = first.copy(transaction = first.transaction.copy(state = TransactionState.Pending))
        val rows = subject.aggregates(activity, listOf(pending, second))

        val updated = subject.aggregates(activity, listOf(first, second))

        assertNotSame(rows.first(), updated.first())
        assertEquals(TransactionState.Confirmed, updated.first().state)
        assertSame(rows.last(), updated.last())
    }

    @Test
    fun droppedTransactionIsNotKeptAlive() {
        subject.aggregates(activity, listOf(first, second))

        val remaining = subject.aggregates(activity, listOf(second))

        assertEquals(1, remaining.size)
    }

    @Test
    fun rowBuiltForOneFilterIsReusedByAnother() {
        val rows = subject.aggregates(activity, listOf(first, second))

        val filtered = subject.aggregates(asset, listOf(second))

        assertSame(rows.last(), filtered.single())
        assertSame(rows.first(), subject.aggregates(activity, listOf(first)).single())
    }
}
