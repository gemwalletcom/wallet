package com.gemwallet.android.data.password

import android.content.Context
import android.util.Log
import java.nio.charset.StandardCharsets.UTF_8
import java.util.Base64

internal class EncryptedKeyValueStore(
    context: Context,
    preferencesFileName: String,
    private val namespace: String,
    private val aeadProvider: AeadProvider,
    private val legacyStore: SecureStringStore,
    private val resetOnInvalidKey: Boolean,
) : SecureStringStore {

    private val sharedPreferences = context.applicationContext.getSharedPreferences(
        preferencesFileName,
        Context.MODE_PRIVATE,
    )

    private val namespacePrefix = "${namespace}_"

    override fun contains(key: String): Boolean = sharedPreferences.contains(storageKey(namespace, key))

    override fun getString(key: String): String? {
        val encryptedValue = sharedPreferences.getString(storageKey(namespace, key), null) ?: return null
        if (!encryptedValue.startsWith(KEYSTORE_VALUE_PREFIX)) {
            val legacyValue = readLegacyValue(key) ?: return null
            runCatching { putString(key, legacyValue) }.onFailure { error ->
                Log.e(TAG, "Keeping legacy value for $namespace, migration failed", error)
            }
            return legacyValue
        }
        val encodedValue = encryptedValue.removePrefix(KEYSTORE_VALUE_PREFIX)
        return try {
            decryptKeystoreValue(key, encodedValue)
        } catch (error: Exception) {
            recoverStoredValue(key, encodedValue, error)
        }
    }

    override fun putString(key: String, value: String) {
        val encodedValue = try {
            encryptValue(key, value, createKeyIfMissing = !hasKeystoreValues())
        } catch (error: Exception) {
            recoverWrite(key, value, error)
        }
        if (!sharedPreferences.edit().putString(storageKey(namespace, key), encodedValue).commit()) {
            throw IllegalStateException("Secure value write failed")
        }
    }

    override fun removeString(key: String): Boolean =
        sharedPreferences.edit().remove(storageKey(namespace, key)).commit()

    private fun encryptValue(key: String, value: String, createKeyIfMissing: Boolean): String {
        val encryptedValue = aeadProvider.encrypt(
            plaintext = value.toByteArray(UTF_8),
            associatedData = associatedData(namespace, key),
            createKeyIfMissing = createKeyIfMissing,
        )
        return KEYSTORE_VALUE_PREFIX + Base64.getEncoder().encodeToString(encryptedValue)
    }

    private fun decryptKeystoreValue(key: String, encodedValue: String): String {
        val decryptedValue = aeadProvider.decrypt(
            ciphertext = Base64.getDecoder().decode(encodedValue),
            associatedData = associatedData(namespace, key),
        )
        return String(decryptedValue, UTF_8)
    }

    private fun readLegacyValue(key: String): String? = try {
        legacyStore.getString(key)
    } catch (error: Exception) {
        if (!resetOnInvalidKey || !(isSecureValueCorruption(error) || isSecureKeyFailure(error))) {
            throw error
        }
        null
    }

    private fun recoverStoredValue(key: String, encodedValue: String, error: Exception): String? {
        if (!resetOnInvalidKey) {
            throw error
        }
        if (isSecureValueCorruption(error)) {
            removeString(key)
            return null
        }
        if (!isSecureKeyFailure(error)) {
            throw error
        }
        // One retry with a rebuilt Aead separates transient keystore faults from an unusable key.
        aeadProvider.refresh()
        return try {
            decryptKeystoreValue(key, encodedValue)
        } catch (retryError: Exception) {
            when {
                isSecureValueCorruption(retryError) -> {
                    removeString(key)
                    null
                }
                isSecureKeyFailure(retryError) -> {
                    resetKeystoreValues()
                    null
                }
                else -> throw retryError
            }
        }
    }

    private fun recoverWrite(key: String, value: String, error: Exception): String {
        if (!resetOnInvalidKey || !isSecureKeyFailure(error)) {
            throw error
        }
        aeadProvider.reset()
        return encryptValue(key, value, createKeyIfMissing = true)
    }

    // Removes only this namespace's keystore-format values: the preferences file is shared
    // with the legacy Tink store, whose values are still decryptable with the Tink keyset.
    private fun resetKeystoreValues() {
        val editor = sharedPreferences.edit()
        sharedPreferences.all.forEach { (preferenceKey, value) ->
            if (isNamespacedKeystoreValue(preferenceKey, value)) {
                editor.remove(preferenceKey)
            }
        }
        if (!editor.commit()) {
            throw IllegalStateException("Secure values reset failed")
        }
        aeadProvider.reset()
    }

    private fun hasKeystoreValues(): Boolean = sharedPreferences.all.any { (preferenceKey, value) ->
        isNamespacedKeystoreValue(preferenceKey, value)
    }

    private fun isNamespacedKeystoreValue(preferenceKey: String, value: Any?): Boolean =
        preferenceKey.startsWith(namespacePrefix) && value is String && value.startsWith(KEYSTORE_VALUE_PREFIX)
}

internal const val KEYSTORE_VALUE_PREFIX = "android-keystore-v1:"

private const val TAG = "EncryptedKeyValueStore"
