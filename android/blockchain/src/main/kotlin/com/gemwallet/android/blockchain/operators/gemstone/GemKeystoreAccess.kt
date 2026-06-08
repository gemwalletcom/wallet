package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.ext.v4KeystorePasswordBytes
import uniffi.gemstone.GemKeystore

internal inline fun <R> withGemKeystore(
    baseDir: String,
    password: String,
    block: (keystore: GemKeystore, passwordBytes: ByteArray) -> R,
): R {
    val passwordBytes = password.v4KeystorePasswordBytes()
    return try {
        GemKeystore(baseDir).use { keystore -> block(keystore, passwordBytes) }
    } finally {
        passwordBytes.fill(0)
    }
}
