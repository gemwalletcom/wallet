package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SecretDataViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    private val walletId = "single_ethereum_0xabc"

    private fun route(vararg extra: Pair<String, Any?>) = SavedStateHandle(mapOf(RouteArgument.WalletId.key to walletId) + extra)

    @Test
    fun `a key wallet's backup exports the wallet secret`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        SecretDataViewModel(service, route(RouteArgument.Type.key to GemWalletSecretKind.PRIVATE_KEY), dispatcher)
        advanceUntilIdle()

        coVerify { service.exportSecret(walletId) }
        coVerify(exactly = 0) { service.exportPrivateKey(any(), any()) }
    }

    @Test
    fun `a selected account exports that chain's key`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        SecretDataViewModel(service, route(RouteArgument.Type.key to GemWalletSecretKind.PRIVATE_KEY, RouteArgument.Chain.key to Chain.Ethereum.string), dispatcher)
        advanceUntilIdle()

        coVerify { service.exportPrivateKey(walletId, Chain.Ethereum.string) }
        coVerify(exactly = 0) { service.exportSecret(any()) }
    }
}
