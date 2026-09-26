package com.gemwallet.android.features.transactions.viewmodels

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionsService

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionsViewModelSyncTest {

    private val session = MutableStateFlow<Session?>(mockSession(wallet = mockWallet()))
    private val service = mockk<GemTransactionsService> {
        every { filterChains(any()) } returns emptyList()
    }
    private val getTransactions = mockk<GetTransactions> {
        every { getTransactions(any(), any()) } returns MutableStateFlow(emptyList<GemTransactionRow>())
        every { stored(any(), any()) } returns emptyList()
    }
    private val getSession = mockk<GetSession>(relaxed = true) {
        every { this@mockk() } returns session
    }

    private val dispatcher = UnconfinedTestDispatcher()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `failed sync is retried on the next screen entry and shows the error while nothing is stored`() = runBlocking {
        val offline = GemServiceException.Gateway("offline")
        coEvery { service.refresh(null, false) } returns GemLoadState.Error(offline)
        val viewModel = createViewModel()

        viewModel.syncIfNeeded()?.join()
        viewModel.syncIfNeeded()?.join()

        coVerify(exactly = 2) { service.refresh(null, false) }
        assertEquals(offline.message, (viewModel.errorRow.value as GemListRow.Error).error.message)
    }

    @Test
    fun `successful sync is not repeated for the same wallet`() = runBlocking {
        coEvery { service.refresh(null, any()) } returns GemLoadState.Data
        val viewModel = createViewModel()

        viewModel.syncIfNeeded()?.join()
        viewModel.syncIfNeeded()?.join()

        coVerify(exactly = 1) { service.refresh(null, any()) }
    }

    @Test
    fun `wallet switch syncs the new wallet`() = runBlocking {
        coEvery { service.refresh(null, any()) } returns GemLoadState.Data
        val viewModel = createViewModel()

        viewModel.syncIfNeeded()?.join()
        session.value = mockSession(wallet = mockWallet(id = WalletId("wallet-2")))
        viewModel.syncIfNeeded()?.join()

        coVerify(exactly = 2) { service.refresh(null, any()) }
    }

    @Test
    fun `a late failure for the previous wallet does not replace the new wallet's result`() = runBlocking {
        val first = CompletableDeferred<GemLoadState>()
        var calls = 0
        coEvery { service.refresh(null, any()) } coAnswers {
            calls++
            if (calls == 1) first.await() else GemLoadState.Data
        }
        val viewModel = createViewModel()

        val previous = viewModel.syncIfNeeded()
        session.value = mockSession(wallet = mockWallet(id = WalletId("wallet-2")))
        viewModel.syncIfNeeded()?.join()
        first.complete(GemLoadState.Error(GemServiceException.Gateway("offline")))
        previous?.join()

        assertNull(viewModel.errorRow.value)
    }

    @Test
    fun `overlapping refreshes keep the spinner until the latest one finishes`() = runBlocking {
        val first = CompletableDeferred<GemLoadState>()
        val second = CompletableDeferred<GemLoadState>()
        var calls = 0
        coEvery { service.refresh(null, any()) } coAnswers {
            calls++
            if (calls == 1) first.await() else second.await()
        }
        val viewModel = createViewModel()

        val earlier = viewModel.refresh()
        val latest = viewModel.refresh()
        first.complete(GemLoadState.Data)
        earlier?.join()
        assertTrue(viewModel.isRefreshing.value)

        second.complete(GemLoadState.Data)
        latest?.join()
        assertFalse(viewModel.isRefreshing.value)
    }

    private fun createViewModel() = TransactionsViewModel(
        getSession = getSession,
        getTransactions = getTransactions,
        service = service,
        observeRefreshInterval = mockk(relaxed = true),
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true),
    )
}
