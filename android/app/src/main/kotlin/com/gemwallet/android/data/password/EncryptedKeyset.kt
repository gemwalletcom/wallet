package com.gemwallet.android.data.password

import android.content.SharedPreferences
import com.gemwallet.android.application.KeystoreKeyChangedException
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.math.hex
import com.google.crypto.tink.Aead
import com.google.crypto.tink.InsecureSecretKeyAccess
import com.google.crypto.tink.KeyTemplate
import com.google.crypto.tink.KeysetHandle
import com.google.crypto.tink.RegistryConfiguration
import com.google.crypto.tink.TinkProtoKeysetFormat
import java.security.GeneralSecurityException
import javax.crypto.AEADBadTagException

internal fun encryptedKeyset(preferences: SharedPreferences, name: String, masterAead: Aead, template: KeyTemplate? = null): KeysetHandle = synchronized(secureStorageLock) {
    val encoded = preferences.getString(name, null)
    val handle = if (encoded == null) {
        checkNotNull(template) { "Secure keyset is missing" }
        KeysetHandle.generateNew(template)
    } else {
        val bytes = encoded.fromHex()
        try {
            return@synchronized TinkProtoKeysetFormat.parseEncryptedKeyset(bytes, masterAead, byteArrayOf(), RegistryConfiguration.get())
        } catch (keystoreError: GeneralSecurityException) {
            try {
                TinkProtoKeysetFormat.parseKeyset(bytes, InsecureSecretKeyAccess.get(), RegistryConfiguration.get())
            } catch (plaintextError: GeneralSecurityException) {
                keystoreError.addSuppressed(plaintextError)
                throw if (keystoreError is AEADBadTagException) KeystoreKeyChangedException(keystoreError) else keystoreError
            }
        } finally {
            bytes.fill(0)
        }
    }
    val encrypted = TinkProtoKeysetFormat.serializeEncryptedKeyset(handle, masterAead, byteArrayOf(), RegistryConfiguration.get())
    check(preferences.putStringDurably(name, encrypted.hex)) { "Secure keyset write failed" }
    handle
}
