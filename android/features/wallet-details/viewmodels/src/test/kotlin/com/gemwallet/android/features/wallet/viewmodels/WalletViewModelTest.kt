package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
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
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemWalletSecret
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class WalletViewModelTest {

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

    private val walletId = "multicoin_0xabc"

    private fun route(vararg extra: Pair<String, Any?>) = SavedStateHandle(mapOf(RouteArgument.WalletId.key to walletId) + extra)

    @Test
    fun `renaming a wallet goes to Core`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        val details: GetWalletDetails = mockk { every { getWallet(any()) } returns flowOf(null) }
        val model = WalletViewModel(details, service, mockk(relaxed = true), route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setWalletName("Savings").join()

        coVerify { service.rename(walletId, "Savings") }
    }

    @Test
    fun `a rename Core refuses shows its error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { rename(any(), any()) } throws IllegalStateException("taken")
        }
        val details: GetWalletDetails = mockk { every { getWallet(any()) } returns flowOf(null) }
        val model = WalletViewModel(details, service, mockk(relaxed = true), route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setWalletName("Savings").join()

        assertEquals("taken", model.error.value)
        model.clearError()
        assertNull(model.error.value)
    }

    @Test
    fun `deleting a wallet hands both callbacks to the case`() = runTest(dispatcher) {
        val delete: DeleteWallet = mockk(relaxed = true)
        val details: GetWalletDetails = mockk { every { getWallet(any()) } returns flowOf(null) }
        val model = WalletViewModel(details, mockk(relaxed = true), delete, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.delete(onBoard = {}, onComplete = {}).join()

        coVerify { delete.deleteWallet(match { it.id == walletId }, any(), any()) }
    }

    @Test
    fun `an export the keystore refuses is captured, not thrown`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { exportSecret(any()) } throws IllegalStateException("locked")
        }
        val model = WalletSecretDataViewModel(
            service,
            route(RouteArgument.Type.key to GemWalletSecretKind.PHRASE),
            dispatcher,
        ).also { models.add(it) }

        val result = model.secret.first { it != null }

        assertTrue(result!!.isFailure)
        assertEquals(GemWalletSecretKind.PHRASE, model.secretKind)
    }

    @Test
    fun `an avatar Core refuses shows an error until it is cleared`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { setAvatarImageUrl(any(), any()) } throws IllegalStateException("no image")
            every { avatarItems(any()) } returns emptyList()
        }
        val details: GetWalletDetails = mockk { every { getWallet(any()) } returns flowOf(null) }
        val nfts: NFTQuery = mockk { every { this@mockk.invoke(any(), any()) } returns flowOf(emptyList()) }
        val model = WalletImageViewModel(details, nfts, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setNftImage("https://example.com/a.png").join()

        assertTrue(model.error.value != null)
        model.clearError()
        assertNull(model.error.value)
    }
}
