package com.gemwallet.android.data.repositories.transactions

import com.wallet.core.primitives.TransactionState
import org.junit.Assert.assertEquals
import org.junit.Test

class TransactionStateChangePolicyTest {

    @Test
    fun completedTransactionIsNotDowngradedToInTransit() {
        assertEquals(
            TransactionState.Confirmed,
            nextTransactionState(
                oldState = TransactionState.Confirmed,
                newState = TransactionState.InTransit,
            ),
        )
    }

    @Test
    fun completedTransactionCanReceiveTerminalState() {
        assertEquals(
            TransactionState.Failed,
            nextTransactionState(
                oldState = TransactionState.Confirmed,
                newState = TransactionState.Failed,
            ),
        )
    }

    @Test
    fun activeTransactionCanMoveToInTransit() {
        assertEquals(
            TransactionState.InTransit,
            nextTransactionState(
                oldState = TransactionState.Pending,
                newState = TransactionState.InTransit,
            ),
        )
    }

    @Test
    fun inTransitTransactionIsNotDowngradedToPending() {
        assertEquals(
            TransactionState.InTransit,
            nextTransactionState(
                oldState = TransactionState.InTransit,
                newState = TransactionState.Pending,
            ),
        )
    }

    @Test
    fun inTransitTransactionCanReceiveTerminalState() {
        assertEquals(
            TransactionState.Confirmed,
            nextTransactionState(
                oldState = TransactionState.InTransit,
                newState = TransactionState.Confirmed,
            ),
        )
    }
}
