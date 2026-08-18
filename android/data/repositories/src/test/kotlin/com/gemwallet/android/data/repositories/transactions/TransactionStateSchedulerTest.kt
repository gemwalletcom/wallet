package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.data.repositories.assets.TransactionPostProcessingService
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.data.service.store.database.entities.mockDbTransactionExtended
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockTransaction
import com.wallet.core.primitives.TransactionState
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import org.junit.Test

class TransactionStateSchedulerTest {

    private val sessionRepository = mockk<SessionRepository> {
        every { session() } returns MutableStateFlow<Session?>(null)
    }
    private val postProcessingService = mockk<TransactionPostProcessingService>(relaxed = true)

    private val subject = TransactionStateScheduler(
        sessionRepository = sessionRepository,
        transactionsDao = mockk<TransactionsDao>(relaxed = true),
        stateService = mockk<TransactionStateService>(),
        postProcessingService = postProcessingService,
    )

    @Test
    fun enteringInTransit_runsPostProcessing() = runBlocking {
        val current = mockTransaction(state = TransactionState.Pending)
        val updated = current.copy(state = TransactionState.InTransit)

        subject.notifyEnteringInTransit(mockDbTransactionExtended(current), mockDbTransactionExtended(updated))

        coVerify(exactly = 1) {
            postProcessingService.processTransactions(
                match { it.single().transaction.state == TransactionState.InTransit },
            )
        }
    }

    @Test
    fun otherTransitions_skipPostProcessing() = runBlocking {
        val current = mockTransaction(state = TransactionState.Pending)
        val updated = current.copy(state = TransactionState.Confirmed)

        subject.notifyEnteringInTransit(mockDbTransactionExtended(current), mockDbTransactionExtended(updated))

        coVerify(exactly = 0) { postProcessingService.processTransactions(any()) }
    }
}
