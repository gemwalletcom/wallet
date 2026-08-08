package com.gemwallet.android.data.password

import com.gemwallet.android.math.hex
import java.nio.charset.StandardCharsets.UTF_8
import java.security.InvalidKeyException
import java.security.KeyStoreException
import java.security.MessageDigest
import java.security.ProviderException
import java.security.UnrecoverableKeyException
import javax.crypto.BadPaddingException

internal interface SecureStringStore {
    fun contains(key: String): Boolean

    fun getString(key: String): String?

    fun putString(key: String, value: String)

    fun removeString(key: String): Boolean
}

internal fun SecureStringStore.getOrMigrate(legacyStore: SecureStringStore, key: String): String? {
    val currentValue = getString(key)
    if (currentValue != null) {
        return currentValue
    }

    val legacyValue = legacyStore.getString(key) ?: return null
    putString(key, legacyValue)
    legacyStore.removeString(key)
    return legacyValue
}

internal fun associatedData(namespace: String, key: String): ByteArray = "$namespace:$key".toByteArray(UTF_8)

internal fun storageKey(namespace: String, key: String): String {
    val digest = MessageDigest.getInstance("SHA-256").digest("$namespace\u0000$key".toByteArray(UTF_8))
    return "${namespace}_${digest.hex}"
}

internal fun isSecureValueCorruption(error: Throwable): Boolean =
    error is BadPaddingException || error is IllegalArgumentException

internal fun isSecureKeyFailure(error: Throwable): Boolean =
    error is InvalidKeyException ||
        error is UnrecoverableKeyException ||
        error is KeyStoreException ||
        error is ProviderException
