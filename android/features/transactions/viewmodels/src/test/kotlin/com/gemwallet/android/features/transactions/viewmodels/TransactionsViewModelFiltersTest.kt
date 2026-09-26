package com.gemwallet.android.features.transactions.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionsFilter
import com.wallet.core.primitives.WalletId
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemTransactionFilter
import uniffi.gemstone.GemTransactionRow
import uniffi.gemstone.GemTransactionsServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class TransactionsViewModelFiltersTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<androidx.lifecycle.ViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val first = mockWallet(id = WalletId("multicoin_0xabc"))
    private val second = mockWallet(id = WalletId("multicoin_0xdef"))

    private val requested = mutableListOf<TransactionsFilter>()

    private fun viewModel(session: MutableStateFlow<Session?>): TransactionsViewModel {
        val service: GemTransactionsServiceInterface = mockk(relaxed = true) {
            every { filterChains(any()) } returns listOf(Chain.Bitcoin.string, Chain.Ethereum.string)
        }
        val transactions: GetTransactions = mockk {
            every { getTransactions(capture(requested), any()) } returns flowOf(emptyList<GemTransactionRow>())
            every { stored(any(), any()) } returns emptyList()
        }
        val getSession: GetSession = mockk { every { this@mockk.invoke() } returns session }
        return TransactionsViewModel(getSession, transactions, service, mockk(relaxed = true), dispatcher).also { models.add(it) }
    }

    @Test
    fun `switching wallets drops the filters the previous wallet had`() = runTest(dispatcher) {
        val session = MutableStateFlow<Session?>(mockSession(wallet = first))
        val model = viewModel(session)
        model.filter.first { it.chains.isNotEmpty() }

        model.setChainsFilter(listOf(Chain.Ethereum))
        model.setTypesFilter(listOf(GemTransactionFilter.SWAPS))
        assertTrue(model.filterView.first { it.isFiltered }.isFiltered)

        session.value = mockSession(wallet = second)

        val cleared = model.filter.first { it.selectedChains.isEmpty() }
        assertTrue(cleared.selectedTypes.isEmpty())
        assertEquals(listOf(Chain.Bitcoin.string, Chain.Ethereum.string), cleared.chains)
    }

    @Test
    fun `a chain selection reaches the activity query`() = runTest(dispatcher) {
        val model = viewModel(MutableStateFlow(mockSession(wallet = first)))
        model.filter.first { it.chains.isNotEmpty() }

        model.setChainsFilter(listOf(Chain.Ethereum))
        advanceUntilIdle()

        assertEquals(listOf(Chain.Ethereum), requested.last().chains)
    }

    @Test
    fun `a details route that is not a transaction id is refused`() {
        assertThrows(IllegalArgumentException::class.java) {
            TransactionViewModel(
                mockk(relaxed = true),
                mockk(relaxed = true),
                mockk(relaxed = true),
                SavedStateHandle(mapOf(RouteArgument.WalletId.key to "wallet", RouteArgument.TransactionId.key to "not-an-id")),
                dispatcher,
                mockk(relaxed = true),
            )
        }
    }
}
