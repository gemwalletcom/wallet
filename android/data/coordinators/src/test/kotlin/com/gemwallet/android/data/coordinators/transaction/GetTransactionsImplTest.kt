package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.values.TransactionsQueryFilter
import com.gemwallet.android.data.services.store.queries.TransactionsQuery
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionListItem
import com.wallet.core.primitives.WalletId
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.concurrent.atomic.AtomicInteger

class GetTransactionsImplTest {

    private val wallet = mockWallet()
    private val subscriptions = AtomicInteger()
    private val transactionsQuery = mockk<TransactionsQuery> {
        every { this@mockk(any(), any(), GemConstants.transactionsListLimit) } returns flow {
            subscriptions.incrementAndGet()
            emit(emptyList<TransactionListItem>())
            awaitCancellation()
        }
    }
    private val getSession = mockk<GetSession> {
        every { this@mockk.invoke() } returns MutableStateFlow(mockSession(wallet = wallet))
    }
    private val getCurrentWalletId = mockk<GetCurrentWalletId> {
        every { this@mockk.invoke() } returns flowOf<WalletId>(wallet.id)
    }

    @Test
    fun screensOnOneFilterShareOneQuery() = runTest {
        val subject = GetTransactionsImpl(getSession, getCurrentWalletId, transactionsQuery, backgroundScope)
        val filters = listOf(TransactionsQueryFilter.Chains(listOf(Chain.Bitcoin)))
        subject.getTransactions(TransactionsQueryFilter.activityDefaults()).first()
        val baseline = subscriptions.get()

        backgroundScope.launch { subject.getTransactions(filters).collect {} }
        runCurrent()
        repeat(3) { subject.getTransactions(filters).first() }

        assertEquals(baseline + 1, subscriptions.get())
    }

    @Test
    fun theActivityScreenSharesTheDefaultObservation() = runTest {
        val subject = GetTransactionsImpl(getSession, getCurrentWalletId, transactionsQuery, backgroundScope)

        subject.getTransactions(TransactionsQueryFilter.activityDefaults()).first()
        subject.getTransactions(TransactionsQueryFilter.activityDefaults()).first()

        assertEquals(1, subscriptions.get())
    }
}
