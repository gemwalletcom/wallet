package com.gemwallet.android.data.adapters.gemstone

import com.gemwallet.android.application.PasswordNotFoundException
import com.gemwallet.android.application.PasswordStore
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertSame
import org.junit.Assert.assertThrows
import org.junit.Test

private const val LEGACY_WALLET_ID = "multicoin_legacy"
private const val FIRST_NEW_WALLET_ID = "multicoin_new_1"
private const val SECOND_NEW_WALLET_ID = "multicoin_new_2"
private const val LEGACY_WALLET_PASSWORD = "0x0102"
private const val APP_PASSWORD = "0x0304"

class KeystorePasswordStoreTest {

    @Test
    fun legacyWalletPasswordRemainsAuthoritative() {
        val passwordStore = TestPasswordStore(
            mutableMapOf(
                LEGACY_WALLET_ID to LEGACY_WALLET_PASSWORD,
                PasswordStore.Keys.Password.key to APP_PASSWORD,
            ),
        )

        val password = GemstoneKeystorePassword(passwordStore).getPassword(LEGACY_WALLET_ID, false)

        assertArrayEquals(byteArrayOf(1, 2), password)
        assertEquals(0, passwordStore.createdPasswords)
    }

    @Test
    fun newWalletsReuseAppPasswordAndReceiveWalletAliases() {
        val passwordStore = TestPasswordStore()
        val keystorePassword = GemstoneKeystorePassword(passwordStore)

        val first = keystorePassword.getPassword(FIRST_NEW_WALLET_ID, true)
        val second = keystorePassword.getPassword(SECOND_NEW_WALLET_ID, true)

        assertArrayEquals(byteArrayOf(3, 4), first)
        assertArrayEquals(first, second)
        assertEquals(1, passwordStore.createdPasswords)
        assertEquals(APP_PASSWORD, passwordStore.getPassword(PasswordStore.Keys.Password.key))
        assertEquals(APP_PASSWORD, passwordStore.getPassword(FIRST_NEW_WALLET_ID))
        assertEquals(APP_PASSWORD, passwordStore.getPassword(SECOND_NEW_WALLET_ID))
    }

    @Test
    fun existingWalletCanUseAppPasswordWithoutCreatingWalletAlias() {
        val passwordStore = TestPasswordStore(
            mutableMapOf(PasswordStore.Keys.Password.key to APP_PASSWORD),
        )

        val password = GemstoneKeystorePassword(passwordStore).getPassword(FIRST_NEW_WALLET_ID, false)

        assertArrayEquals(byteArrayOf(3, 4), password)
        assertFalse(passwordStore.contains(FIRST_NEW_WALLET_ID))
    }

    @Test
    fun existingWalletWithoutAnyPasswordFailsClosed() {
        assertThrows(PasswordNotFoundException::class.java) {
            GemstoneKeystorePassword(TestPasswordStore()).getPassword(LEGACY_WALLET_ID, false)
        }
    }

    @Test
    fun storageFailureDoesNotFallBackToAppPassword() {
        val storageError = IllegalStateException("secure storage unavailable")
        val passwordStore = TestPasswordStore(
            passwords = mutableMapOf(PasswordStore.Keys.Password.key to APP_PASSWORD),
            readFailures = mapOf(LEGACY_WALLET_ID to storageError),
        )

        val thrown = assertThrows(IllegalStateException::class.java) {
            GemstoneKeystorePassword(passwordStore).getPassword(LEGACY_WALLET_ID, false)
        }

        assertSame(storageError, thrown)
    }

    @Test
    fun aliasWriteFailureDoesNotReturnAppPassword() {
        val writeError = IllegalStateException("secure storage write failed")
        val passwordStore = TestPasswordStore(
            passwords = mutableMapOf(PasswordStore.Keys.Password.key to APP_PASSWORD),
            writeFailure = writeError,
        )

        val thrown = assertThrows(IllegalStateException::class.java) {
            GemstoneKeystorePassword(passwordStore).getPassword(FIRST_NEW_WALLET_ID, true)
        }

        assertSame(writeError, thrown)
    }

    private class TestPasswordStore(
        private val passwords: MutableMap<String, String> = mutableMapOf(),
        private val readFailures: Map<String, RuntimeException> = emptyMap(),
        private val writeFailure: RuntimeException? = null,
    ) : PasswordStore {
        var createdPasswords = 0
            private set

        override fun getOrCreatePassword(key: String): String = passwords.getOrPut(key) {
            createdPasswords += 1
            APP_PASSWORD
        }

        override fun removePassword(key: String): Boolean = passwords.remove(key) != null

        override fun getPassword(key: String): String {
            readFailures[key]?.let { throw it }
            return passwords[key] ?: throw PasswordNotFoundException()
        }

        override fun putPassword(key: String, password: String) {
            writeFailure?.let { throw it }
            passwords[key] = password
        }

        fun contains(key: String): Boolean = passwords.containsKey(key)
    }
}
