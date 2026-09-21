package com.gemwallet.android

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.features.setup_wallet.viewmodels.SetupWalletViewModel
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SetupWalletViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<SetupWalletViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val wallet = mockWallet(id = "multicoin_0xabc", name = "Main Wallet")

    private fun viewModel(service: GemWalletServiceInterface = mockk(relaxed = true)): SetupWalletViewModel {
        val getWallet: GetWallet = mockk { every { this@mockk.invoke(any()) } returns flowOf(wallet) }
        return SetupWalletViewModel(WalletId(wallet.id.id), getWallet, service, dispatcher).also { models.add(it) }
    }

    @Test
    fun `the scene opens on the wallet name and the row Core builds`() = runTest(dispatcher) {
        val model = viewModel()

        val state = model.uiState.first { it.row != null }

        assertEquals("Main Wallet", state.walletName)
        assertEquals("Main Wallet", state.row?.name)
    }

    @Test
    fun `renaming shows the new name and writes it through`() = runTest(dispatcher) {
        val renamed = CompletableDeferred<Pair<String, String>>()
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { rename(any(), any()) } answers { renamed.complete(firstArg<String>() to secondArg<String>()) }
        }
        val model = viewModel(service)
        model.uiState.first { it.row != null }

        model.onNameChange("Savings")

        assertEquals("Savings", model.uiState.first { it.walletName == "Savings" }.walletName)
        assertEquals(wallet.id.id to "Savings", renamed.await())
    }

    @Test
    fun `a rename Core refuses shows an error until it is cleared`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { rename(any(), any()) } throws IllegalStateException("taken")
        }
        val model = viewModel(service)
        model.uiState.first { it.row != null }

        model.onNameChange("Savings")

        assertNotNull(model.uiState.first { it.error != null }.error)
        model.clearError()
        assertNull(model.uiState.first { it.error == null }.error)
    }
}
