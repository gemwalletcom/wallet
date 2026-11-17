package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class WalletDetailViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private val walletId = "multicoin_0xabc"

    private val route = SavedStateHandle(mapOf(RouteArgument.WalletId.key to walletId))

    private val walletQuery: WalletQuery = mockk { every { this@mockk(any()) } returns flowOf(null) }

    @Test
    fun `renaming a wallet goes to Core`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        val model = WalletDetailViewModel(walletQuery, service, route, dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setWalletName("Savings")
        advanceUntilIdle()

        coVerify { service.rename(walletId, "Savings") }
    }

    @Test
    fun `renames are saved in the order the name was typed`() = runTest(dispatcher) {
        val saved = mutableListOf<String>()
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { rename(walletId, "Sa") } coAnswers {
                delay(100)
                saved.add("Sa")
            }
            coEvery { rename(walletId, "Sav") } coAnswers { saved.add("Sav") }
        }
        val model = WalletDetailViewModel(walletQuery, service, route, dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setWalletName("Sa")
        runCurrent()
        model.setWalletName("Sav")
        advanceUntilIdle()

        assertEquals(listOf("Sa", "Sav"), saved)
    }

    @Test
    fun `a rename Core refuses shows its error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { rename(any(), any()) } throws IllegalStateException("taken")
        }
        val model = WalletDetailViewModel(walletQuery, service, route, dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setWalletName("Savings")
        advanceUntilIdle()

        assertEquals("taken", model.error.value)
        model.clearError()
        assertNull(model.error.value)
    }

    @Test
    fun `a delete publishes what Core reports`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet(walletId) } returns GemWalletDeletion.LAST_WALLET_DELETED
        }
        val model = WalletDetailViewModel(walletQuery, service, route, dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.delete().join()

        assertEquals(GemWalletDeletion.LAST_WALLET_DELETED, model.deletion.value)
    }

    @Test
    fun `a delete Core refuses keeps the user in place and shows the unknown error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet(walletId) } throws GemServiceException.Core("keystore delete failed")
        }
        val context: Context = mockk { every { getString(R.string.errors_unknown) } returns "Unknown error" }
        val model = WalletDetailViewModel(walletQuery, service, route, dispatcher, context).also { models.add(it) }

        model.delete().join()

        assertEquals("Unknown error", model.error.value)
        assertNull(model.deletion.value)
    }

    @Test
    fun `a delete failing outside Core keeps the user in place without an error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet(walletId) } throws IllegalStateException("keystore delete failed")
        }
        val model = WalletDetailViewModel(walletQuery, service, route, dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.delete().join()

        assertNull(model.error.value)
        assertNull(model.deletion.value)
    }
}
