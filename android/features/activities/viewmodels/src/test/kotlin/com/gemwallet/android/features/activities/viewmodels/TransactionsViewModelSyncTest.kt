package com.gemwallet.android.features.activities.viewmodels

import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.SyncTransactions
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.transaction.aggregates.TransactionDataAggregate
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetConfigService

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionsViewModelSyncTest {

    private val session = MutableStateFlow<Session?>(mockSession(wallet = mockWallet()))
    private val syncTransactions = mockk<SyncTransactions>()
    private val getTransactions = mockk<GetTransactions> {
        every { getTransactions(any()) } returns MutableStateFlow(emptyList<TransactionDataAggregate>())
        every { transactions() } returns MutableStateFlow(emptyList())
    }
    private val getSession = mockk<GetSession>(relaxed = true) {
        every { this@mockk() } returns session
    }

    @Before
    fun setUp() = Dispatchers.setMain(UnconfinedTestDispatcher())

    @After
    fun tearDown() = Dispatchers.resetMain()

    @Test
    fun `failed sync is retried on the next screen entry`() = runBlocking {
        coEvery { syncTransactions.syncTransactions(any()) } returns false
        val viewModel = createViewModel()

        viewModel.syncIfNeeded()?.join()
        viewModel.syncIfNeeded()?.join()

        coVerify(exactly = 2) { syncTransactions.syncTransactions(any()) }
    }

    @Test
    fun `successful sync is not repeated for the same wallet`() = runBlocking {
        coEvery { syncTransactions.syncTransactions(any()) } returns true
        val viewModel = createViewModel()

        viewModel.syncIfNeeded()?.join()
        viewModel.syncIfNeeded()?.join()

        coVerify(exactly = 1) { syncTransactions.syncTransactions(any()) }
    }

    @Test
    fun `wallet switch syncs the new wallet`() = runBlocking {
        coEvery { syncTransactions.syncTransactions(any()) } returns true
        val viewModel = createViewModel()

        viewModel.syncIfNeeded()?.join()
        session.value = mockSession(wallet = mockWallet(id = "wallet-2"))
        viewModel.syncIfNeeded()?.join()

        coVerify { syncTransactions.syncTransactions(match { it.id.id == "wallet-2" }) }
    }

    private fun createViewModel() = TransactionsViewModel(
        getSession = getSession,
        getTransactions = getTransactions,
        syncTransactions = syncTransactions,
        assetConfig = GemAssetConfigService(),
    )
}
