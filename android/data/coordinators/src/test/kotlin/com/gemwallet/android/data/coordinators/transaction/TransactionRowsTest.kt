package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.transactions.values.TransactionsQueryFilter
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionId
import com.gemwallet.android.testkit.mockTransactionListItem
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.WalletId
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test

class TransactionRowsTest {

    private val subject = TransactionRows()
    private val wallet = WalletId("wallet")
    private val activity = TransactionsQueryFilter.activityDefaults()
    private val asset = listOf(TransactionsQueryFilter.Chains(listOf(Chain.Bitcoin)))

    private val first = mockTransactionListItem(mockTransaction(id = mockTransactionId(hash = "first"), state = TransactionState.Confirmed))
    private val second = mockTransactionListItem(mockTransaction(id = mockTransactionId(hash = "second")))

    @Test
    fun unchangedTransactionsKeepTheirRow() {
        val items = listOf(first, second)

        val rows = subject.rows(wallet, activity, items)

        assertEquals(rows, subject.rows(wallet, activity, items))
        assertSame(rows.first(), subject.rows(wallet, activity, items).first())
    }

    @Test
    fun changedTransactionGetsANewRow() {
        val pending = first.copy(transaction = first.transaction.copy(state = TransactionState.Pending))
        val rows = subject.rows(wallet, activity, listOf(pending, second))

        val updated = subject.rows(wallet, activity, listOf(first, second))

        assertNotSame(rows.first(), updated.first())
        assertEquals(TransactionState.Confirmed, updated.first().state.toPrimitives())
        assertSame(rows.last(), updated.last())
    }

    @Test
    fun droppedTransactionIsNotKeptAlive() {
        subject.rows(wallet, activity, listOf(first, second))

        val remaining = subject.rows(wallet, activity, listOf(second))

        assertEquals(1, remaining.size)
    }

    @Test
    fun rowBuiltForOneFilterIsReusedByAnother() {
        val rows = subject.rows(wallet, activity, listOf(first, second))

        val filtered = subject.rows(wallet, asset, listOf(second))

        assertSame(rows.last(), filtered.single())
        assertSame(rows.first(), subject.rows(wallet, activity, listOf(first)).single())
    }

    @Test
    fun storedRowsBelongToTheirWallet() {
        val rows = subject.rows(wallet, activity, listOf(first))

        assertEquals(rows, subject.stored(wallet, activity))
        assertTrue(subject.stored(WalletId("other"), activity).isEmpty())
    }

    @Test
    fun visitingManyFiltersKeepsOnlyTheRecentOnesAndTheActivity() {
        subject.rows(wallet, activity, listOf(first))
        val chains = Chain.entries.take(RecentFilters.LIMIT * 2).map { listOf(TransactionsQueryFilter.Chains(listOf(it))) }
        chains.forEach { subject.rows(wallet, it, listOf(second)) }

        assertTrue(subject.stored(wallet, chains.first()).isEmpty())
        assertEquals(1, subject.stored(wallet, chains.last()).size)
        assertEquals(1, subject.stored(wallet, activity).size)
    }
}
