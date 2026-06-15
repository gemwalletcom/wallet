package com.gemwallet.android.data.password

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import java.security.SecureRandom

private const val TEST_WALLET_KEY = "wallet-1"
private const val LEGACY_PASSWORD = "legacy-password"
private const val ENCRYPTED_PASSWORD = "encrypted-password"
private const val NEW_PASSWORD = "new-password"
private const val GENERATED_PASSWORD = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

class TinkPasswordStoreTest {

    private val encryptedStore = InMemorySecureStringStore()
    private val legacyStore = InMemorySecureStringStore()
    private val passwordStore = TinkPasswordStore(
        encryptedStore = encryptedStore,
        legacyStore = legacyStore,
        random = IncrementingSecureRandom(),
    )

    @Test
    fun getPassword_migratesLegacyValue() {
        legacyStore.putString(TEST_WALLET_KEY, LEGACY_PASSWORD)

        assertEquals(LEGACY_PASSWORD, passwordStore.getPassword(TEST_WALLET_KEY))
        assertEquals(LEGACY_PASSWORD, encryptedStore.getString(TEST_WALLET_KEY))
        assertEquals(null, legacyStore.getString(TEST_WALLET_KEY))
    }

    @Test
    fun getPassword_prefersEncryptedValueAndRemovesLegacyValue() {
        encryptedStore.putString(TEST_WALLET_KEY, ENCRYPTED_PASSWORD)
        legacyStore.putString(TEST_WALLET_KEY, LEGACY_PASSWORD)

        assertEquals(ENCRYPTED_PASSWORD, passwordStore.getPassword(TEST_WALLET_KEY))
        assertEquals(null, legacyStore.getString(TEST_WALLET_KEY))
    }

    @Test
    fun putPassword_writesEncryptedValueAndRemovesLegacyValue() {
        legacyStore.putString(TEST_WALLET_KEY, LEGACY_PASSWORD)

        passwordStore.putPassword(TEST_WALLET_KEY, NEW_PASSWORD)

        assertEquals(NEW_PASSWORD, encryptedStore.getString(TEST_WALLET_KEY))
        assertEquals(null, legacyStore.getString(TEST_WALLET_KEY))
    }

    @Test
    fun createPassword_writesGeneratedPasswordAndRemovesLegacyValue() {
        legacyStore.putString(TEST_WALLET_KEY, LEGACY_PASSWORD)

        val password = passwordStore.createPassword(TEST_WALLET_KEY)

        assertEquals(GENERATED_PASSWORD, password)
        assertEquals(password, encryptedStore.getString(TEST_WALLET_KEY))
        assertEquals(null, legacyStore.getString(TEST_WALLET_KEY))
    }

    @Test
    fun removePassword_removesBothStores() {
        encryptedStore.putString(TEST_WALLET_KEY, ENCRYPTED_PASSWORD)
        legacyStore.putString(TEST_WALLET_KEY, LEGACY_PASSWORD)

        assertTrue(passwordStore.removePassword(TEST_WALLET_KEY))
        assertEquals(null, encryptedStore.getString(TEST_WALLET_KEY))
        assertEquals(null, legacyStore.getString(TEST_WALLET_KEY))
    }

    @Test
    fun getPassword_missingValueFailsClosed() {
        assertThrows(IllegalStateException::class.java) {
            passwordStore.getPassword(TEST_WALLET_KEY)
        }
    }

    private class InMemorySecureStringStore : SecureStringStore {
        private val values = mutableMapOf<String, String>()

        override fun contains(key: String): Boolean = values.containsKey(key)

        override fun getString(key: String): String? = values[key]

        override fun putString(key: String, value: String) {
            values[key] = value
        }

        override fun removeString(key: String): Boolean {
            values.remove(key)
            return true
        }
    }

    private class IncrementingSecureRandom : SecureRandom() {
        override fun nextBytes(bytes: ByteArray) {
            bytes.indices.forEach { index -> bytes[index] = index.toByte() }
        }
    }
}
