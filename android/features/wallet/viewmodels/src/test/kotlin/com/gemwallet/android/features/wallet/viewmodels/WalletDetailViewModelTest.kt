package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockGemChainRow
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemAvatarList
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemPrivateKeyAccount
import uniffi.gemstone.GemPrivateKeyScope
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletSecret
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class WalletDetailViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<androidx.lifecycle.ViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private val walletId = "multicoin_0xabc"

    private fun route(vararg extra: Pair<String, Any?>) = SavedStateHandle(mapOf(RouteArgument.WalletId.key to walletId) + extra)

    private val walletQuery: WalletQuery = mockk { every { this@mockk(any()) } returns flowOf(null) }

    @Test
    fun `renaming a wallet goes to Core`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true)
        val model = WalletDetailViewModel(walletQuery, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

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
        val model = WalletDetailViewModel(walletQuery, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

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
        val model = WalletDetailViewModel(walletQuery, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setWalletName("Savings")
        advanceUntilIdle()

        assertEquals("taken", model.error.value)
        model.clearError()
        assertNull(model.error.value)
    }

    @Test
    fun `deleting a wallet while others remain completes without onboarding`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet(walletId) } returns GemWalletDeletion.WALLETS_REMAINING
        }
        val onBoard = mockk<() -> Unit>(relaxed = true)
        val onComplete = mockk<() -> Unit>(relaxed = true)
        val model = WalletDetailViewModel(walletQuery, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.delete(onBoard, onComplete).join()

        verify(exactly = 1) { onComplete() }
        verify(exactly = 0) { onBoard() }
    }

    @Test
    fun `deleting the last wallet returns to onboarding`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet(walletId) } returns GemWalletDeletion.LAST_WALLET_DELETED
        }
        val onBoard = mockk<() -> Unit>(relaxed = true)
        val onComplete = mockk<() -> Unit>(relaxed = true)
        val model = WalletDetailViewModel(walletQuery, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.delete(onBoard, onComplete).join()

        verify(exactly = 1) { onBoard() }
        verify(exactly = 0) { onComplete() }
    }

    @Test
    fun `a delete Core refuses keeps the user in place and shows its error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet(walletId) } throws GemServiceException.Platform("keystore delete failed")
        }
        val onBoard = mockk<() -> Unit>(relaxed = true)
        val onComplete = mockk<() -> Unit>(relaxed = true)
        val model = WalletDetailViewModel(walletQuery, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.delete(onBoard, onComplete).join()

        assertEquals("keystore delete failed", model.error.value)
        verify(exactly = 0) { onBoard() }
        verify(exactly = 0) { onComplete() }
    }

    @Test
    fun `a delete failing outside Core keeps the user in place without an error`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { deleteWallet(walletId) } throws IllegalStateException("keystore delete failed")
        }
        val onBoard = mockk<() -> Unit>(relaxed = true)
        val onComplete = mockk<() -> Unit>(relaxed = true)
        val model = WalletDetailViewModel(walletQuery, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.delete(onBoard, onComplete).join()

        assertNull(model.error.value)
        verify(exactly = 0) { onBoard() }
        verify(exactly = 0) { onComplete() }
    }

    @Test
    fun `an export the keystore refuses is captured, not thrown`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk(relaxed = true) {
            coEvery { exportSecret(any()) } throws IllegalStateException("locked")
        }
        val model = SecretDataViewModel(
            service,
            route(RouteArgument.Type.key to GemWalletSecretKind.PHRASE),
            dispatcher,
        ).also { models.add(it) }

        val result = model.secret.first { it != null }

        assertTrue(result!!.isFailure)
        assertEquals(GemWalletSecretKind.PHRASE, model.secretKind)
    }

    @Test
    fun `a phrase route exports the secret`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { exportSecret(walletId) } returns GemWalletSecret.Words(listOf("abandon"))
        }
        val model = SecretDataViewModel(service, route(RouteArgument.Type.key to GemWalletSecretKind.PHRASE), dispatcher).also { models.add(it) }

        advanceUntilIdle()

        assertEquals(GemWalletSecret.Words(listOf("abandon")), model.secret.value!!.getOrThrow())
    }

    @Test
    fun `a private key route exports the key of its chain`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { exportPrivateKey(walletId, Chain.Ethereum.string) } returns GemWalletSecret.PrivateKey("0xkey", GemPrivateKeyScope.ONE_CHAIN)
        }
        val model = SecretDataViewModel(
            service,
            route(RouteArgument.Type.key to GemWalletSecretKind.PRIVATE_KEY, RouteArgument.Chain.key to Chain.Ethereum.string),
            dispatcher,
        ).also { models.add(it) }

        advanceUntilIdle()

        assertEquals(GemWalletSecret.PrivateKey("0xkey", GemPrivateKeyScope.ONE_CHAIN), model.secret.value!!.getOrThrow())
        assertEquals(GemWalletSecretKind.PRIVATE_KEY, model.secretKind)
    }

    @Test
    fun `private key accounts load from Core`() = runTest(dispatcher) {
        val accounts = listOf(GemPrivateKeyAccount(mockGemChainRow(chain = Chain.Ethereum.string), "0x12...34"))
        val service: GemWalletServiceInterface = mockk {
            coEvery { privateKeyAccounts(walletId) } returns accounts
        }
        val model = PrivateKeyAccountsViewModel(service, route(), dispatcher).also { models.add(it) }

        advanceUntilIdle()

        assertEquals(accounts, model.accounts.value!!.getOrThrow())
    }

    @Test
    fun `private key accounts Core refuses are captured, not thrown`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { privateKeyAccounts(walletId) } throws IllegalStateException("locked")
        }
        val model = PrivateKeyAccountsViewModel(service, route(), dispatcher).also { models.add(it) }

        advanceUntilIdle()

        assertTrue(model.accounts.value!!.isFailure)
    }

    @Test
    fun `an avatar Core refuses shows an error until it is cleared`() = runTest(dispatcher) {
        val service: GemWalletServiceInterface = mockk {
            coEvery { setAvatarImageUrl(any(), any()) } throws IllegalStateException("no image")
            every { avatarList(any()) } returns GemAvatarList(emptyList(), GemListPhase.Rows)
        }
        val nfts: NFTQuery = mockk { every { this@mockk.invoke(any(), any()) } returns flowOf(emptyList()) }
        val model = WalletImageViewModel(walletQuery, nfts, service, route(), dispatcher, mockk(relaxed = true)).also { models.add(it) }

        model.setNftImage("https://example.com/a.png").join()

        assertTrue(model.error.value != null)
        model.clearError()
        assertNull(model.error.value)
    }
}
