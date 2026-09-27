package com.gemwallet.android.features.wallets.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.testkit.mockGemWalletRow
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import io.mockk.verifyOrder
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletSection
import uniffi.gemstone.GemWalletSectionKind
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.GemWalletSubtitle

@OptIn(ExperimentalCoroutinesApi::class)
class WalletsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val wallets = MutableStateFlow<List<GemWalletSection>>(emptyList())
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
    fun `a pin toggles the wallet Core listed`() = runTest(dispatcher) {
        val pinned = mockGemWalletRow(id = "pinned", name = "Pinned", subtitle = GemWalletSubtitle.Multicoin, placeholder = GemWalletPlaceholder.Multicoin, isPinned = true)
        wallets.value = listOf(GemWalletSection(GemWalletSectionKind.PINNED, listOf(pinned)))
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        val model = viewModel(service)

        model.togglePin(WalletId("pinned")).join()
        model.togglePin(WalletId("gone")).join()

        assertEquals(wallets.value, model.sections.value)
        coVerify(exactly = 1) { service.setPinned("pinned", false) }
        coVerify(exactly = 0) { service.setPinned("gone", any()) }
    }

    @Test
    fun `selecting a wallet makes it current before closing the list`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        val onSelected = mockk<() -> Unit>(relaxed = true)
        val model = viewModel(service)

        model.selectWallet(WalletId("second"), onSelected).join()

        verifyOrder {
            service.setCurrentWalletId("second")
            onSelected()
        }
    }

    @Test
    fun `deleting a wallet while others remain stays on the list`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet("wallet-1") } returns GemWalletDeletion.WALLETS_REMAINING
        }
        val onBoard = mockk<() -> Unit>(relaxed = true)

        viewModel(service).deleteWallet(WalletId("wallet-1"), onBoard).join()

        coVerify(exactly = 1) { service.deleteWallet("wallet-1") }
        verify(exactly = 0) { onBoard() }
    }

    @Test
    fun `deleting the last wallet returns to onboarding`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet("wallet-1") } returns GemWalletDeletion.LAST_WALLET_DELETED
        }
        val onBoard = mockk<() -> Unit>(relaxed = true)

        viewModel(service).deleteWallet(WalletId("wallet-1"), onBoard).join()

        verify(exactly = 1) { onBoard() }
    }

    @Test
    fun `a delete Core refuses keeps the user in place and shows its error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet("wallet-1") } throws IllegalStateException("keystore delete failed")
        }
        val onBoard = mockk<() -> Unit>(relaxed = true)
        val model = viewModel(service)

        model.deleteWallet(WalletId("wallet-1"), onBoard).join()

        assertEquals("keystore delete failed", model.error.value)
        verify(exactly = 0) { onBoard() }
    }

    private fun viewModel(service: GemWalletServiceInterface = mockk(relaxed = true)): WalletsViewModel {
        val getAllWallets: GetAllWallets = mockk { every { getAllWallets() } returns wallets }
        return WalletsViewModel(
            getAllWallets = getAllWallets,
            service = service,
            ioDispatcher = dispatcher,
            context = mockk<Context>(relaxed = true),
        ).also { models.add(it) }
    }
}
