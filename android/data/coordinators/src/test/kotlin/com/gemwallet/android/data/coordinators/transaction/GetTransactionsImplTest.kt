package com.gemwallet.android.data.coordinators.transaction

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.gemstone.stores.GemstoneTransactionStore
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
    private val store = mockk<GemstoneTransactionStore> {
        every { observeTransactions(any(), any()) } returns flow {
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
        val subject = GetTransactionsImpl(getSession, getCurrentWalletId, store, backgroundScope)
        val filters = listOf(TransactionsRequestFilter.Chains(listOf(Chain.Bitcoin)))
        subject.getTransactions(TransactionsRequestFilter.activityDefaults()).first()
        val baseline = subscriptions.get()

        backgroundScope.launch { subject.getTransactions(filters).collect {} }
        runCurrent()
        repeat(3) { subject.getTransactions(filters).first() }

        assertEquals(baseline + 1, subscriptions.get())
    }

    @Test
    fun theActivityScreenSharesTheDefaultObservation() = runTest {
        val subject = GetTransactionsImpl(getSession, getCurrentWalletId, store, backgroundScope)

        subject.getTransactions(TransactionsRequestFilter.activityDefaults()).first()
        subject.getTransactions(TransactionsRequestFilter.activityDefaults()).first()

        assertEquals(1, subscriptions.get())
    }
}
