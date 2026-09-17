package com.gemwallet.android

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetPendingTransactionsCount
import com.gemwallet.android.application.wallet_connect.cases.IsWalletConnectEnabled
import com.gemwallet.android.features.main.viewmodels.MainScreenViewModel
import com.gemwallet.android.testkit.mockSession
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
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
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

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

    private fun viewModel(
        pending: Int?,
        coordinator: PendingNavigationCoordinator = mockk(relaxed = true),
    ): MainScreenViewModel {
        val session: GetSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(mockSession()) }
        val connect: IsWalletConnectEnabled = mockk { every { isWalletConnectEnabled() } returns true }
        val counts: GetPendingTransactionsCount = mockk { every { getPendingTransactionsCount() } returns flowOf(pending) }
        return MainScreenViewModel(session, coordinator, connect, counts).also { models.add(it) }
    }

    @Test
    fun `no pending transactions means no badge`() = runTest(dispatcher) {
        val model = viewModel(pending = 0)

        assertNull(model.pendingTxCount.first())
    }

    @Test
    fun `pending transactions are counted on the badge`() = runTest(dispatcher) {
        val model = viewModel(pending = 3)

        assertEquals("3", model.pendingTxCount.first { it != null })
    }

    @Test
    fun `a scanned code goes to the navigation coordinator`() = runTest(dispatcher) {
        val coordinator: PendingNavigationCoordinator = mockk(relaxed = true)
        val model = viewModel(pending = 0, coordinator = coordinator)

        model.onScan("bitcoin:bc1q")

        verify { coordinator.handleScan("bitcoin:bc1q") }
    }
}
