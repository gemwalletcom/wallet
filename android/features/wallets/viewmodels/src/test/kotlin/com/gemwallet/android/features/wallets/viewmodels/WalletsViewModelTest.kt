package com.gemwallet.android.features.wallets.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.testkit.mockGemWalletRow
import com.gemwallet.android.testkit.mockWalletDataAggregate
import com.wallet.core.primitives.WalletId
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class WalletsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val wallets = MutableStateFlow<List<WalletDataAggregate>>(emptyList())
    private val models = mutableListOf<WalletsViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    @Test
    fun `the pinned wallets come first and each row is already rendered`() = runTest(dispatcher) {
        wallets.value = listOf(
            mockWalletDataAggregate(mockGemWalletRow(id = "pinned", name = "Pinned", isPinned = true)),
            mockWalletDataAggregate(mockGemWalletRow(id = "unpinned", name = "Plain")),
            mockWalletDataAggregate(mockGemWalletRow(id = "second-pinned", name = "Second", isPinned = true)),
        )
        val model = viewModel()
        advanceUntilIdle()

        val state = model.uiState.value
        assertEquals(listOf("pinned", "second-pinned"), state.pinned.map { it.row.id })
        assertEquals(listOf("unpinned"), state.unpinned.map { it.row.id })
        assertEquals(listOf("Pinned", "Second"), state.pinned.map { it.row.name })
        assertEquals("Plain", state.name(WalletId("unpinned")))
        assertEquals("", state.name(WalletId("gone")))
    }

    @Test
    fun `no wallets leaves both sections empty`() = runTest(dispatcher) {
        val model = viewModel()
        advanceUntilIdle()

        assertTrue(model.uiState.value.pinned.isEmpty())
        assertTrue(model.uiState.value.unpinned.isEmpty())
    }

    private fun viewModel(): WalletsViewModel {
        val getAllWallets: GetAllWallets = mockk { every { getAllWallets() } returns wallets }
        return WalletsViewModel(
            getAllWallets = getAllWallets,
            setCurrentWallet = mockk(relaxed = true),
            service = mockk(relaxed = true),
            deleteWallet = mockk(relaxed = true),
            ioDispatcher = dispatcher,
            context = mockk<Context>(relaxed = true),
        ).also { models.add(it) }
    }
}
