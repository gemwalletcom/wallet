package com.gemwallet.android.features.fiat_connect.viewmodels

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.FiatTransactionsQuery
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.FiatTransactionAssetData
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemFiatQuoteServiceInterface
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemServiceException

@OptIn(ExperimentalCoroutinesApi::class)
class FiatTransactionsViewModelTest {

    private val dispatcher = UnconfinedTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @Test
    fun `a failed refresh with nothing stored shows the error instead of the empty state`() = runBlocking {
        val offline = GemServiceException.Gateway("offline")
        val service = mockk<GemFiatQuoteServiceInterface> {
            coEvery { refreshTransactions() } returns GemLoadState.Data
        }
        val viewModel = createViewModel(service)
        viewModel.refresh().join()
        assertTrue(viewModel.phase.value is GemListPhase.Empty)

        coEvery { service.refreshTransactions() } returns GemLoadState.Error(offline)
        viewModel.refresh().join()

        assertEquals(offline.message, (viewModel.phase.value as GemListPhase.Error).error.message)
    }

    private fun createViewModel(service: GemFiatQuoteServiceInterface): FiatTransactionsViewModel {
        val getSession = mockk<GetSession> {
            every { this@mockk() } returns MutableStateFlow(mockSession(wallet = mockWallet()))
        }
        val query = mockk<FiatTransactionsQuery> {
            every { this@mockk(any()) } returns flowOf(emptyList<FiatTransactionAssetData>())
        }
        return FiatTransactionsViewModel(getSession = getSession, fiatTransactionsQuery = query, service = service, ioDispatcher = dispatcher)
    }
}
