@file:Suppress("DEPRECATION")

package com.gemwallet.android.data.password

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.hex
import com.google.crypto.tink.proto.EncryptedKeyset
import com.google.crypto.tink.shaded.protobuf.ByteString
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.nio.charset.StandardCharsets.UTF_8
import java.security.GeneralSecurityException
import java.security.MessageDigest
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
    fun encryptedStore_roundTripsAndRejectsMismatchedAssociatedData() {
        val sourceKey = "source-key"
        val targetKey = "target-key"
        val store = TinkEncryptedKeyValueStore.create(
            context = context,
            config = TEST_STORE_CONFIG,
        )

        store.putString(sourceKey, "secret-value")

        assertTrue(store.contains(sourceKey))
        assertEquals("secret-value", store.getString(sourceKey))

        val rawPreferences = context.getSharedPreferences(TEST_PREFERENCES_FILE_NAME, Context.MODE_PRIVATE)
        val encryptedValue = rawPreferences.getString(storageKey(TEST_NAMESPACE, sourceKey), null)
        assertNotNull(encryptedValue)
        assertTrue(rawPreferences.edit().putString(storageKey(TEST_NAMESPACE, targetKey), encryptedValue).commit())

        try {
            store.getString(targetKey)
            fail("Expected mismatched associated data to fail decryption")
        } catch (_: GeneralSecurityException) {
        }
    }

    @Test
    fun encryptedStore_doesNotResetInvalidKeysetByDefault() {
        val originalStore = TinkEncryptedKeyValueStore.create(
            context = context,
            config = TEST_STORE_CONFIG,
        )
        originalStore.putString("original-key", "original-value")
        mockCorruptKeyset()

        val reopenedStore = TinkEncryptedKeyValueStore.create(
            context = context,
            config = TEST_STORE_CONFIG,
        )

        try {
            reopenedStore.putString("new-key", "new-value")
            fail("Expected invalid keyset to fail decryption")
        } catch (_: BadPaddingException) {
        }
        assertTrue(reopenedStore.contains("original-key"))
    }

    @Test
    fun encryptedStore_resetsInvalidKeysetWhenConfigured() {
        val originalStore = TinkEncryptedKeyValueStore.create(
            context = context,
            config = RECOVERABLE_TEST_STORE_CONFIG,
        )
        originalStore.putString("original-key", "original-value")
        mockCorruptKeyset()

        val reopenedStore = TinkEncryptedKeyValueStore.create(
            context = context,
            config = RECOVERABLE_TEST_STORE_CONFIG,
        )
        reopenedStore.putString("new-key", "new-value")

        assertFalse(reopenedStore.contains("original-key"))
        assertEquals("new-value", reopenedStore.getString("new-key"))
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
    }

    private fun mockCorruptKeyset() {
        val preferences = context.getSharedPreferences(TEST_KEYSET_PREFERENCES_FILE_NAME, Context.MODE_PRIVATE)
        val serializedKeyset = preferences.getString(TEST_KEYSET_NAME, null)!!.fromHex()
        val keyset = EncryptedKeyset.parseFrom(serializedKeyset)
        val encryptedKeyset = keyset.encryptedKeyset.toByteArray()
        encryptedKeyset[encryptedKeyset.lastIndex] = (encryptedKeyset.last().toInt() xor 1).toByte()
        val corruptedKeyset = keyset.toBuilder()
            .setEncryptedKeyset(ByteString.copyFrom(encryptedKeyset))
            .build()
            .toByteArray()
            .hex
        assertTrue(preferences.edit().putString(TEST_KEYSET_NAME, corruptedKeyset).commit())
    }

    private fun storageKey(namespace: String, key: String): String {
        val digest = MessageDigest.getInstance("SHA-256").digest("$namespace\u0000$key".toByteArray(UTF_8))
        return "${namespace}_${digest.joinToString(separator = "") { "%02x".format(it.toInt() and 0xff) }}"
    }

    companion object {
        private const val TEST_PREFERENCES_FILE_NAME = "instrumented_secure_values"
        private const val TEST_NAMESPACE = "instrumented_secure_namespace"
        private const val TEST_KEYSET_NAME = "instrumented_secure_values_keyset"
        private const val TEST_KEYSET_PREFERENCES_FILE_NAME = "instrumented_secure_values_keyset_prefs"
        private const val TEST_MASTER_KEY_ALIAS = "instrumented_secure_values_master_key"
        private val TEST_STORE_CONFIG = TinkStoreConfig(
            preferencesFileName = TEST_PREFERENCES_FILE_NAME,
            namespace = TEST_NAMESPACE,
            keysetName = TEST_KEYSET_NAME,
            keysetPreferencesFileName = TEST_KEYSET_PREFERENCES_FILE_NAME,
            masterKeyAlias = TEST_MASTER_KEY_ALIAS,
            resetOnInvalidKeyset = false,
        )
        private val RECOVERABLE_TEST_STORE_CONFIG = TEST_STORE_CONFIG.copy(resetOnInvalidKeyset = true)
    }
}
