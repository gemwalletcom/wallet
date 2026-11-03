package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockGemWalletRow
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
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

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    private val wallets = MutableStateFlow<List<GemWalletSection>>(emptyList())
    private val models = mutableListOf<WalletsViewModel>()

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    @Test
    fun `a pin flips the pinned state the row carries`() = runTest(dispatcher) {
        val pinned = mockGemWalletRow(id = "pinned", name = "Pinned", subtitle = GemWalletSubtitle.Multicoin, placeholder = GemWalletPlaceholder.Multicoin, isPinned = true)
        val unpinned = mockGemWalletRow(id = "unpinned", name = "Unpinned", subtitle = GemWalletSubtitle.Multicoin, placeholder = GemWalletPlaceholder.Multicoin)
        wallets.value = listOf(GemWalletSection(GemWalletSectionKind.PINNED, listOf(pinned)), GemWalletSection(GemWalletSectionKind.WALLETS, listOf(unpinned)))
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        val model = viewModel(service)

        model.togglePin(pinned).join()
        model.togglePin(unpinned).join()

        assertEquals(wallets.value, model.sections.value)
        coVerify(exactly = 1) { service.setPinned("pinned", false) }
        coVerify(exactly = 1) { service.setPinned("unpinned", true) }
    }

    @Test
    fun `selecting a wallet makes it current before closing the list`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        val model = viewModel(service)

        model.selectWallet(mockGemWalletRow(id = "second")).join()

        coVerify(exactly = 1) { service.setCurrentWalletId("second") }
        assertTrue(model.isSelected.value)
    }

    @Test
    fun `a wallet that cannot be made current keeps the list open`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { setCurrentWalletId("second") } throws IllegalStateException("locked")
        }
        val model = viewModel(service)

        model.selectWallet(mockGemWalletRow(id = "second")).join()

        assertFalse(model.isSelected.value)
    }

    @Test
    fun `a delete publishes what Core reports`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet("wallet-1") } returns GemWalletDeletion.LAST_WALLET_DELETED
        }
        val model = viewModel(service)

        model.deleteWallet(mockGemWalletRow(id = "wallet-1")).join()

        coVerify(exactly = 1) { service.deleteWallet("wallet-1") }
        assertEquals(GemWalletDeletion.LAST_WALLET_DELETED, model.deletion.value)
    }

    @Test
    fun `a delete Core refuses keeps the user in place and shows its error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet("wallet-1") } throws IllegalStateException("keystore delete failed")
        }
        val model = viewModel(service)

        model.deleteWallet(mockGemWalletRow(id = "wallet-1")).join()

        assertEquals("keystore delete failed", model.error.value)
        assertNull(model.deletion.value)
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
