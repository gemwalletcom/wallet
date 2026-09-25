package com.gemwallet.android.features.activities.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
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

    private fun viewModel(session: MutableStateFlow<Session?>): TransactionsViewModel {
        val service: GemTransactionsServiceInterface = mockk(relaxed = true) {
            every { filterChains(any()) } returns emptyList()
        }
        val transactions: GetTransactions = mockk {
            every { getTransactions(any()) } returns flowOf(emptyList<GemTransactionRow>())
            every { stored(any()) } returns emptyList()
        }
        val getSession: GetSession = mockk { every { this@mockk.invoke() } returns session }
        return TransactionsViewModel(getSession, transactions, service, mockk(relaxed = true), dispatcher, mockk(relaxed = true)).also { models.add(it) }
    }

    @Test
    fun `switching wallets drops the filters the previous wallet had`() = runTest(dispatcher) {
        val session = MutableStateFlow<Session?>(mockSession(wallet = first))
        val model = viewModel(session)
        model.walletId.first { it != null }

        model.setChainsFilter(listOf(Chain.Ethereum))
        model.setTypesFilter(listOf(GemTransactionFilter.SWAPS))
        assertEquals(listOf(Chain.Ethereum), model.chainsFilter.value)

        session.value = mockSession(wallet = second)

        assertTrue(model.chainsFilter.first { it.isEmpty() }.isEmpty())
        assertTrue(model.typeFilter.first { it.isEmpty() }.isEmpty())
    }

    @Test
    fun `clearing a filter leaves the other one alone`() = runTest(dispatcher) {
        val model = viewModel(MutableStateFlow(mockSession(wallet = first)))

        model.setChainsFilter(listOf(Chain.Ethereum))
        model.setTypesFilter(listOf(GemTransactionFilter.SWAPS))

        model.clearChainsFilter()

        assertTrue(model.chainsFilter.value.isEmpty())
        assertEquals(listOf(GemTransactionFilter.SWAPS), model.typeFilter.value)
    }

    @Test
    fun `a details route that is not a transaction id is refused`() {
        assertThrows(IllegalArgumentException::class.java) {
            TransactionDetailsViewModel(mockk(relaxed = true), mockk(relaxed = true), mockk(relaxed = true), SavedStateHandle(mapOf(RouteArgument.TransactionId.key to "not-an-id")), dispatcher, mockk(relaxed = true))
        }
    }
}
