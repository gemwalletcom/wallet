package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockGemChainRow
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemPrivateKeyAccount
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class PrivateKeyAccountsViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ViewModel>()
    private val walletId = "multicoin_0xabc"
    private val route = SavedStateHandle(mapOf(RouteArgument.WalletId.key to walletId))

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    @Test
    fun `private key accounts load from Core`() = runTest(dispatcher) {
        val accounts = listOf(
            GemPrivateKeyAccount(mockGemChainRow(chain = Chain.Bitcoin.string), "bc1q...0wlh"),
            GemPrivateKeyAccount(mockGemChainRow(chain = Chain.Ethereum.string), "0x12...34"),
        )
        val service: GemWalletServiceInterface = mockk {
            coEvery { privateKeyAccounts(walletId) } returns accounts
        }
        val model = PrivateKeyAccountsViewModel(service, route, dispatcher).also { models.add(it) }

        advanceUntilIdle()

        assertEquals(accounts, model.accounts.value!!.getOrThrow())
    }

    @Test
    fun `private key accounts Core refuses are captured, not thrown`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { privateKeyAccounts(walletId) } throws IllegalStateException("locked")
        }
        val model = PrivateKeyAccountsViewModel(service, route, dispatcher).also { models.add(it) }

        advanceUntilIdle()

        assertTrue(model.accounts.value!!.isFailure)
    }
}
