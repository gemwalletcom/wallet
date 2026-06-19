package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.LoadPrivateDataOperator
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.testkit.TEST_PHRASE
import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class GetWalletSecretDataImplTest {

    private val walletsRepository = mockk<WalletsRepository>()
    private val passwordStore = mockk<PasswordStore>()
    private val loadPrivateDataOperator = mockk<LoadPrivateDataOperator>()

    private val subject = GetWalletSecretDataImpl(
        walletsRepository = walletsRepository,
        passwordStore = passwordStore,
        loadPrivateDataOperator = loadPrivateDataOperator,
    )

    @Test
    fun success_returnsPhrase_notError() = runTest {
        val wallet = mockWallet()
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { passwordStore.getPassword(wallet.id.id) } returns "0xdeadbeef"
        coEvery { loadPrivateDataOperator(wallet, "0xdeadbeef") } returns TEST_PHRASE

        val value = subject.getSecretData(wallet.id).first()

        assertFalse(value.isError)
        assertEquals(12, value.phrase().size)
    }

    @Test
    fun keystoreFailure_surfacesError_notBlankPhrase() = runTest {
        val wallet = mockWallet()
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { passwordStore.getPassword(wallet.id.id) } throws IllegalStateException("Password not found")

        val value = subject.getSecretData(wallet.id).first()

        assertTrue(value.isError)
        assertTrue(value.phrase().isEmpty())
    }

    @Test
    fun cancellation_isNotMappedToError() = runTest {
        val wallet = mockWallet()
        every { walletsRepository.getWallet(wallet.id) } returns flowOf(wallet)
        every { passwordStore.getPassword(wallet.id.id) } throws CancellationException("cancelled")

        val value = subject.getSecretData(wallet.id).firstOrNull()

        // Cancellation is rethrown (structured concurrency), never converted into a false isError emission.
        assertFalse("cancellation must not surface as isError", value?.isError == true)
    }
}
