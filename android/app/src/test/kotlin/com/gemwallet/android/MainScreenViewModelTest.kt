package com.gemwallet.android

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.TransactionsCountQuery
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.main.viewmodels.MainScreenViewModel
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletId
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.pendingActivityFilters

@OptIn(ExperimentalCoroutinesApi::class)
class MainScreenViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<MainScreenViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val first = mockWallet(id = WalletId("first"))
    private val second = mockWallet(id = WalletId("second"))
    private val currentWalletId = MutableStateFlow(first.id)
    private val counts: TransactionsCountQuery = mockk {
        every { this@mockk(first.id, pendingActivityFilters().toPrimitives()) } returns flowOf(3)
        every { this@mockk(second.id, pendingActivityFilters().toPrimitives()) } returns flowOf(1)
    }

    private fun viewModel(coordinator: PendingNavigationCoordinator = mockk(relaxed = true), counts: TransactionsCountQuery = this.counts): MainScreenViewModel {
        val getCurrentWalletId: GetCurrentWalletId = mockk { every { this@mockk.invoke() } returns currentWalletId }
        return MainScreenViewModel(getCurrentWalletId, coordinator, counts).also { models.add(it) }
    }

    @Test
    fun `the badge starts at 0 before the count arrives`() = runTest(dispatcher) {
        val model = viewModel(counts = mockk { every { this@mockk(any(), any()) } returns emptyFlow() })
        advanceUntilIdle()

        assertEquals(0, model.pendingTxCount.value)
    }

    @Test
    fun `the badge counts the pending transactions of the current wallet`() = runTest(dispatcher) {
        val model = viewModel()
        advanceUntilIdle()
        assertEquals(3, model.pendingTxCount.value)

        currentWalletId.value = second.id
        advanceUntilIdle()

        assertEquals(1, model.pendingTxCount.value)
    }

    @Test
    fun `a scanned code goes to the navigation coordinator`() = runTest(dispatcher) {
        val coordinator: PendingNavigationCoordinator = mockk(relaxed = true)
        val model = viewModel(coordinator = coordinator)

        model.onScan("bitcoin:bc1q")

        verify { coordinator.pendScan("bitcoin:bc1q") }
    }
}
