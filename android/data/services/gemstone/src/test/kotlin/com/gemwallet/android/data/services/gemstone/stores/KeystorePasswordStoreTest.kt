package com.gemwallet.android.data.services.gemstone.stores

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
    fun sharedPasswordIsCreatedOnceAndReusedWithoutWalletAliases() {
        val passwordStore = TestPasswordStore()
        val keystorePassword = GemstoneKeystorePassword(passwordStore)

        val first = keystorePassword.getPassword(true)
        val second = keystorePassword.getPassword(true)

        assertEquals(APP_PASSWORD, first)
        assertEquals(first, second)
        assertEquals(1, passwordStore.createdPasswords)
        assertFalse(passwordStore.contains(FIRST_NEW_WALLET_ID))
    }

    @Test
    fun readingTheSharedPasswordFailsClosedWhenItIsMissing() {
        assertThrows(PasswordNotFoundException::class.java) {
            GemstoneKeystorePassword(TestPasswordStore()).getPassword(false)
        }
    }

    @Test
    fun walletPasswordIsReadForLegacyEntriesAndNullOtherwise() {
        val passwordStore = TestPasswordStore(
            mutableMapOf(
                LEGACY_WALLET_ID to LEGACY_WALLET_PASSWORD,
                PasswordStore.Keys.Password.key to APP_PASSWORD,
            ),
        )
        val keystorePassword = GemstoneKeystorePassword(passwordStore)

        assertEquals(LEGACY_WALLET_PASSWORD, keystorePassword.getWalletPassword(LEGACY_WALLET_ID))
        assertEquals(null, keystorePassword.getWalletPassword(FIRST_NEW_WALLET_ID))

        keystorePassword.deleteWalletPassword(LEGACY_WALLET_ID)

        assertEquals(null, keystorePassword.getWalletPassword(LEGACY_WALLET_ID))
        assertEquals(APP_PASSWORD, passwordStore.getPassword(PasswordStore.Keys.Password.key))
    }

    @Test
    fun storageFailureIsNotSwallowedAsAMissingWalletPassword() {
        val storageError = IllegalStateException("secure storage unavailable")
        val passwordStore = TestPasswordStore(
            passwords = mutableMapOf(PasswordStore.Keys.Password.key to APP_PASSWORD),
            readFailures = mapOf(LEGACY_WALLET_ID to storageError),
        )

        val thrown = assertThrows(IllegalStateException::class.java) {
            GemstoneKeystorePassword(passwordStore).getWalletPassword(LEGACY_WALLET_ID)
        }

        assertSame(storageError, thrown)
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

        override fun hasPassword(key: String): Boolean {
            readFailures[key]?.let { throw it }
            return passwords.containsKey(key)
        }

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
