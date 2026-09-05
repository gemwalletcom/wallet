package com.gemwallet.android.blockchain.services

import com.gemwallet.android.ext.toGem
import android.util.Log
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.testkit.mockWallet
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertSame
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemSignerInput

class KeystoreTransactionSignerTest {

    @Before
    fun setUp() {
        mockkStatic(Log::class)
        every { Log.e(any(), any()) } returns 0
    }

    @After
    fun tearDown() = unmockkStatic(Log::class)

    @Test
    fun passwordFailureIsLoggedWithoutMessageAndRethrown() = runBlocking {
        val passwordStore = mockk<PasswordStore>()
        val failure = IllegalStateException("sensitive-marker")
        every { passwordStore.getPassword(PasswordStore.Keys.Password.key) } throws failure
        val signer = KeystoreTransactionSigner(baseDir = "unused", passwordStore = passwordStore)
        val wallet = mockWallet()

        val error = runCatching {
            signer.sign(wallet.toGem(), mockk<GemSignerInput>())
        }.exceptionOrNull()

        assertSame(failure, error)
        verify(exactly = 1) { passwordStore.getPassword(PasswordStore.Keys.Password.key) }
        verify(exactly = 0) { passwordStore.getPassword(wallet.id.id) }
        verify(exactly = 1) { Log.e("KeystoreTransactionSigner", "keystore transaction signing failed (IllegalStateException)") }
    }

    @Test
    fun cancellationIsRethrownWithoutLogging() = runBlocking {
        val passwordStore = mockk<PasswordStore>()
        val cancellation = CancellationException("cancelled")
        every { passwordStore.getPassword(PasswordStore.Keys.Password.key) } throws cancellation
        val signer = KeystoreTransactionSigner(baseDir = "unused", passwordStore = passwordStore)

        val error = runCatching {
            signer.sign(mockWallet().toGem(), mockk<GemSignerInput>())
        }.exceptionOrNull()

        assertSame(cancellation, error)
        verify(exactly = 0) { Log.e(any(), any()) }
    }
}
