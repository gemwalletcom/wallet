package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockTransactionExtended
import com.gemwallet.android.testkit.mockTransactionId
import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotSame
import org.junit.Assert.assertSame
import org.junit.Test
import uniffi.gemstone.GemAddressService

class TransactionRowsTest {

    private val subject = TransactionRows(GemAddressService())

    private fun transaction(hash: String, state: TransactionState = TransactionState.Confirmed) =
        mockTransactionExtended(transaction = mockTransaction(id = mockTransactionId(hash = hash), state = state))

    @Test
    fun unchangedTransactionsKeepTheirRow() {
        val items = listOf(transaction("first"), transaction("second"))

        val rows = subject.aggregates(items)

        assertEquals(rows, subject.aggregates(items))
        assertSame(rows.first(), subject.aggregates(items).first())
    }

    @Test
    fun changedTransactionGetsANewRow() {
        val pending = transaction("first", TransactionState.Pending)
        val rows = subject.aggregates(listOf(pending, transaction("second")))

        val updated = subject.aggregates(listOf(transaction("first", TransactionState.Confirmed), transaction("second")))

        assertNotSame(rows.first(), updated.first())
        assertEquals(TransactionState.Confirmed, updated.first().state)
        assertSame(rows.last(), updated.last())
    }

    @Test
    fun droppedTransactionIsNotKeptAlive() {
        val first = transaction("first")
        subject.aggregates(listOf(first, transaction("second")))

        val remaining = subject.aggregates(listOf(transaction("second")))

        assertEquals(1, remaining.size)
    }
}
