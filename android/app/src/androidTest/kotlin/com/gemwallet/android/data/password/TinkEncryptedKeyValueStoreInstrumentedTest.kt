@file:Suppress("DEPRECATION")

package com.gemwallet.android.data.password

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.hex
import com.google.crypto.tink.integration.android.AndroidKeystore
import com.google.crypto.tink.proto.EncryptedKeyset
import com.google.crypto.tink.shaded.protobuf.ByteString
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.security.GeneralSecurityException
import javax.crypto.BadPaddingException

@RunWith(AndroidJUnit4::class)
class TinkEncryptedKeyValueStoreInstrumentedTest {

    private val context = ApplicationProvider.getApplicationContext<Context>()

    @Before
    fun setUp() {
        cleanup()
    }

    @After
    fun tearDown() {
        cleanup()
    }

    @Test
    fun passwordStore_migratesLegacyEncryptedPreferencesValue() {
        val key = "instrumented_wallet_password"
        legacyPreferences().edit().putString(key, "legacy-password").commit()

        val passwordStore = TinkPasswordStore(context)

        assertEquals("legacy-password", passwordStore.getPassword(key))
        assertFalse(legacyPreferences().contains(key))
        assertEquals("legacy-password", passwordStore.getPassword(key))
    }

    @Test
    fun gemPreferences_migratesLegacyEncryptedPreferencesValue() {
        val key = "instrumented_gem_preference"
        legacyPreferences().edit().putString(key, "legacy-preference").commit()

        val preferences = TinkGemPreferences(context)

        assertEquals("legacy-preference", preferences.get(key))
        assertFalse(legacyPreferences().contains(key))
        assertEquals("legacy-preference", preferences.get(key))
    }

    @Test
    fun storageKeyDerivationAndValueFormatArePinned() {
        assertEquals(PINNED_STORAGE_KEY, storageKey(TEST_NAMESPACE, PINNED_KEY))
        assertEquals(PINNED_VALUE_PREFIX, KEYSTORE_VALUE_PREFIX)
    }

    @Test
    fun encryptedStore_persistsUnderPinnedStorageKeyAndPrefix() {
        val store = encryptedStore()

        store.putString(PINNED_KEY, PINNED_VALUE)

        val storedValue = testPreferences().getString(PINNED_STORAGE_KEY, null)
        assertNotNull(storedValue)
        assertTrue(storedValue!!.startsWith(PINNED_VALUE_PREFIX))
        assertEquals(PINNED_VALUE, store.getString(PINNED_KEY))
    }

    @Test
    fun tinkStore_rejectsMismatchedAssociatedData() {
        val store = legacyTinkStore()
        store.putString(SOURCE_KEY, SECRET_VALUE)

        copyStoredValue(SOURCE_KEY, TARGET_KEY)

        assertThrows(GeneralSecurityException::class.java) {
            store.getString(TARGET_KEY)
        }
    }

    @Test
    fun encryptedStore_removesOnlyValueWithMismatchedAssociatedData() {
        val store = encryptedStore()
        store.putString(SOURCE_KEY, SECRET_VALUE)
        store.putString("other-key", "other-value")

        copyStoredValue(SOURCE_KEY, TARGET_KEY)

        assertNull(store.getString(TARGET_KEY))
        assertFalse(store.contains(TARGET_KEY))
        assertEquals(SECRET_VALUE, store.getString(SOURCE_KEY))
        assertEquals("other-value", store.getString("other-key"))
    }

    @Test
    fun encryptedStore_removesOnlyCorruptValue() {
        val store = encryptedStore()
        store.putString("corrupt-key", "corrupt-value")
        store.putString("healthy-key", "healthy-value")

        corruptStoredValue("corrupt-key")

        assertNull(store.getString("corrupt-key"))
        assertFalse(store.contains("corrupt-key"))
        assertEquals("healthy-value", store.getString("healthy-key"))
    }

    @Test
    fun tinkStore_doesNotResetInvalidKeyset() {
        val originalStore = legacyTinkStore()
        originalStore.putString(ORIGINAL_KEY, ORIGINAL_VALUE)
        mockCorruptKeyset()

        val reopenedStore = legacyTinkStore()

        assertThrows(BadPaddingException::class.java) {
            reopenedStore.putString(NEW_KEY, NEW_VALUE)
        }
        assertTrue(reopenedStore.contains(ORIGINAL_KEY))
    }

    @Test
    fun encryptedStore_writesKeystoreCiphertextWithoutCreatingTinkKeyset() {
        val store = encryptedStore()

        store.putString(NEW_KEY, NEW_VALUE)

        assertEquals(NEW_VALUE, store.getString(NEW_KEY))
        assertKeystoreValue(NEW_KEY)
        assertFalse(keysetPreferences().contains(TEST_KEYSET_NAME))
    }

    @Test
    fun encryptedStore_migratesTinkValueOnRead() {
        legacyTinkStore().putString(LEGACY_KEY, LEGACY_VALUE)
        val store = encryptedStore()

        assertEquals(LEGACY_VALUE, store.getString(LEGACY_KEY))
        assertKeystoreValue(LEGACY_KEY)

        mockCorruptKeyset()
        assertEquals(LEGACY_VALUE, encryptedStore().getString(LEGACY_KEY))
    }

    @Test
    fun encryptedStore_newWriteDoesNotLoadCorruptTinkKeyset() {
        legacyTinkStore().putString(LEGACY_KEY, LEGACY_VALUE)
        mockCorruptKeyset()

        val store = encryptedStore()
        store.putString(NEW_KEY, NEW_VALUE)

        assertTrue(store.contains(LEGACY_KEY))
        assertEquals(NEW_VALUE, store.getString(NEW_KEY))

        assertNull(store.getString(LEGACY_KEY))
        assertTrue(store.contains(LEGACY_KEY))
    }

    @Test
    fun encryptedStore_resetsValuesWhenDirectKeyIsMissing() {
        val store = encryptedStore()
        store.putString(ORIGINAL_KEY, ORIGINAL_VALUE)
        AndroidKeystore.deleteKey(TEST_AEAD_KEY_ALIAS)

        val reopenedStore = encryptedStore()

        assertNull(reopenedStore.getString(ORIGINAL_KEY))
        assertFalse(reopenedStore.contains(ORIGINAL_KEY))

        reopenedStore.putString(NEW_KEY, NEW_VALUE)
        assertEquals(NEW_VALUE, reopenedStore.getString(NEW_KEY))
    }

    @Test
    fun encryptedStore_resetPreservesLegacyTinkValues() {
        legacyTinkStore().putString(LEGACY_KEY, LEGACY_VALUE)
        val store = encryptedStore()
        store.putString("keystore-key", "keystore-value")
        AndroidKeystore.deleteKey(TEST_AEAD_KEY_ALIAS)

        val reopenedStore = encryptedStore()

        assertNull(reopenedStore.getString("keystore-key"))
        assertEquals(LEGACY_VALUE, reopenedStore.getString(LEGACY_KEY))
    }

    @Test
    fun encryptedStore_writeRecoversWhenDirectKeyIsMissing() {
        val store = encryptedStore()
        store.putString(ORIGINAL_KEY, ORIGINAL_VALUE)
        AndroidKeystore.deleteKey(TEST_AEAD_KEY_ALIAS)

        val reopenedStore = encryptedStore()
        reopenedStore.putString(NEW_KEY, NEW_VALUE)

        assertEquals(NEW_VALUE, reopenedStore.getString(NEW_KEY))
        assertNull(reopenedStore.getString(ORIGINAL_KEY))
    }

    private fun legacyPreferences() =
        EncryptedSharedPreferences.create(
            context,
            LEGACY_PREFERENCES_FILE_NAME,
            MasterKey.Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build(),
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )

    private fun cleanup() {
        listOf(
            LEGACY_PREFERENCES_FILE_NAME,
            PASSWORD_STORE_PREFERENCES_FILE_NAME,
            PASSWORD_STORE_KEYSET_PREFERENCES_FILE_NAME,
            GEM_PREFERENCES_FILE_NAME,
            GEM_PREFERENCES_KEYSET_FILE_NAME,
            TEST_PREFERENCES_FILE_NAME,
            TEST_KEYSET_PREFERENCES_FILE_NAME,
        ).forEach(context::deleteSharedPreferences)
        listOf(TEST_MASTER_KEY_ALIAS, TEST_AEAD_KEY_ALIAS).forEach { alias ->
            runCatching { AndroidKeystore.deleteKey(alias) }
        }
    }

    private fun legacyTinkStore(): TinkEncryptedKeyValueStore = TinkEncryptedKeyValueStore.create(
        context = context,
        config = TEST_STORE_CONFIG,
    )

    private fun encryptedStore(): EncryptedKeyValueStore = EncryptedKeyValueStore(
        context = context,
        preferencesFileName = TEST_PREFERENCES_FILE_NAME,
        namespace = TEST_NAMESPACE,
        aeadProvider = AeadProvider(keyAlias = TEST_AEAD_KEY_ALIAS),
        legacyStore = legacyTinkStore(),
        resetOnInvalidKey = true,
    )

    private fun assertKeystoreValue(key: String) {
        val value = testPreferences().getString(storageKey(TEST_NAMESPACE, key), null)
        assertTrue(value?.startsWith(KEYSTORE_VALUE_PREFIX) == true)
    }

    private fun copyStoredValue(fromKey: String, toKey: String) {
        val preferences = testPreferences()
        val encryptedValue = preferences.getString(storageKey(TEST_NAMESPACE, fromKey), null)!!
        preferences.edit().putString(storageKey(TEST_NAMESPACE, toKey), encryptedValue).commit()
    }

    private fun corruptStoredValue(key: String) {
        val preferences = testPreferences()
        val storedValue = preferences.getString(storageKey(TEST_NAMESPACE, key), null)!!
        val corruptIndex = KEYSTORE_VALUE_PREFIX.length + 5
        val corruptChar = if (storedValue[corruptIndex] == 'A') 'B' else 'A'
        val corruptValue = storedValue.substring(0, corruptIndex) + corruptChar + storedValue.substring(corruptIndex + 1)
        preferences.edit().putString(storageKey(TEST_NAMESPACE, key), corruptValue).commit()
    }

    private fun mockCorruptKeyset() {
        val preferences = keysetPreferences()
        val serializedKeyset = preferences.getString(TEST_KEYSET_NAME, null)!!.fromHex()
        val keyset = EncryptedKeyset.parseFrom(serializedKeyset)
        val encryptedKeyset = keyset.encryptedKeyset.toByteArray()
        encryptedKeyset[encryptedKeyset.lastIndex] = (encryptedKeyset.last().toInt() xor 1).toByte()
        val corruptedKeyset = keyset.toBuilder()
            .setEncryptedKeyset(ByteString.copyFrom(encryptedKeyset))
            .build()
            .toByteArray()
            .hex
        preferences.edit().putString(TEST_KEYSET_NAME, corruptedKeyset).commit()
    }

    private fun testPreferences() =
        context.getSharedPreferences(TEST_PREFERENCES_FILE_NAME, Context.MODE_PRIVATE)

    private fun keysetPreferences() =
        context.getSharedPreferences(TEST_KEYSET_PREFERENCES_FILE_NAME, Context.MODE_PRIVATE)

    companion object {
        private const val TEST_PREFERENCES_FILE_NAME = "instrumented_secure_values"
        private const val TEST_NAMESPACE = "instrumented_secure_namespace"
        private const val TEST_KEYSET_NAME = "instrumented_secure_values_keyset"
        private const val TEST_KEYSET_PREFERENCES_FILE_NAME = "instrumented_secure_values_keyset_prefs"
        private const val TEST_MASTER_KEY_ALIAS = "instrumented_secure_values_master_key"
        private const val TEST_AEAD_KEY_ALIAS = "instrumented_secure_values_aead_v1"

        private const val PINNED_KEY = "pinned-key"
        private const val PINNED_VALUE = "pinned-value"
        private const val PINNED_VALUE_PREFIX = "android-keystore-v1:"
        private const val LEGACY_KEY = "legacy-key"
        private const val LEGACY_VALUE = "legacy-value"
        private const val NEW_KEY = "new-key"
        private const val NEW_VALUE = "new-value"
        private const val ORIGINAL_KEY = "original-key"
        private const val ORIGINAL_VALUE = "original-value"
        private const val SOURCE_KEY = "source-key"
        private const val TARGET_KEY = "target-key"
        private const val SECRET_VALUE = "secret-value"

        // SHA-256("instrumented_secure_namespace" + NUL + "pinned-key")
        private const val PINNED_STORAGE_KEY =
            "instrumented_secure_namespace_b83b25f1f343059346ac9d719965a0cb4a6a5abbd677321224783b7b80b65270"

        private val TEST_STORE_CONFIG = TinkStoreConfig(
            preferencesFileName = TEST_PREFERENCES_FILE_NAME,
            namespace = TEST_NAMESPACE,
            keysetName = TEST_KEYSET_NAME,
            keysetPreferencesFileName = TEST_KEYSET_PREFERENCES_FILE_NAME,
            masterKeyAlias = TEST_MASTER_KEY_ALIAS,
        )
    }
}
