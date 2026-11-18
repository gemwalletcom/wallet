package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
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
import uniffi.gemstone.GemPrivateKeyScope
import uniffi.gemstone.GemWalletSecret
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SecretDataViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<ViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private fun route(walletId: String, vararg extra: Pair<String, Any?>) = SavedStateHandle(mapOf(RouteArgument.WalletId.key to walletId) + extra)

    @Test
    fun `a phrase route exports the secret`() = runTest(dispatcher) {
        val walletId = "multicoin_0xabc"
        val service: GemWalletServiceInterface = mockk {
            coEvery { exportSecret(walletId) } returns GemWalletSecret.Words(listOf("abandon"))
        }
        val model = SecretDataViewModel(service, route(walletId, RouteArgument.Type.key to GemWalletSecretKind.PHRASE), dispatcher).also { models.add(it) }

        advanceUntilIdle()

        assertEquals(GemWalletSecret.Words(listOf("abandon")), model.secret.value!!.getOrThrow())
        assertEquals(GemWalletSecretKind.PHRASE, model.secretKind)
    }

    @Test
    fun `a key wallet's backup exports the wallet secret`() = runTest(dispatcher) {
        val walletId = "privateKey_ethereum_0xabc"
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        SecretDataViewModel(service, route(walletId, RouteArgument.Type.key to GemWalletSecretKind.PRIVATE_KEY), dispatcher).also { models.add(it) }

        advanceUntilIdle()

        coVerify { service.exportSecret(walletId) }
        coVerify(exactly = 0) { service.exportPrivateKey(any(), any()) }
    }

    @Test
    fun `a selected account exports that chain's key`() = runTest(dispatcher) {
        val walletId = "multicoin_0xabc"
        val secret = GemWalletSecret.PrivateKey("0xkey", GemPrivateKeyScope.SharedChains("EVM"))
        val service: GemWalletServiceInterface = mockk {
            coEvery { exportPrivateKey(walletId, Chain.Ethereum.string) } returns secret
        }
        val model = SecretDataViewModel(
            service,
            route(walletId, RouteArgument.Type.key to GemWalletSecretKind.PRIVATE_KEY, RouteArgument.Chain.key to Chain.Ethereum.string),
            dispatcher,
        ).also { models.add(it) }

        advanceUntilIdle()

        assertEquals(secret, model.secret.value!!.getOrThrow())
        assertEquals(GemWalletSecretKind.PRIVATE_KEY, model.secretKind)
        coVerify(exactly = 0) { service.exportSecret(any()) }
    }

    @Test
    fun `an export the keystore refuses is captured, not thrown`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { exportSecret(any()) } throws IllegalStateException("locked")
        }
        val model = SecretDataViewModel(service, route("multicoin_0xabc", RouteArgument.Type.key to GemWalletSecretKind.PHRASE), dispatcher).also { models.add(it) }

        advanceUntilIdle()

        assertTrue(model.secret.value!!.isFailure)
    }
}
